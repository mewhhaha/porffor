use porffor_aot_wasm::{decode_heap_bigint_decimal, WasmRuntimeValueTag};
use porffor_front::{parse, ParseDiagnostic, ParseGoal, ParseOptions, SourceUnit};
use porffor_ir::{lower, IrDiagnostic, IrDiagnosticKind, ProgramIr, ValueKind};
use sha2::{Digest, Sha256};
use wasmparser::{Parser as WasmParser, Payload as WasmPayload};
use wasmtime::{
    Caller as WasmtimeCaller, Config as WasmtimeConfig, Engine as WasmtimeEngine,
    Extern as WasmtimeExtern, Linker as WasmtimeLinker, Module as WasmtimeModule, OptLevel,
    RegallocAlgorithm, Store as WasmtimeStore, StoreLimits as WasmtimeStoreLimits,
    StoreLimitsBuilder as WasmtimeStoreLimitsBuilder, Trap as WasmtimeTrap, Val as WasmtimeVal,
};

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

mod cache;

pub use cache::{cache_status, prune_caches, CacheDirectoryStatus, CachePruneReport, CacheStatus};

const WASM_RESULT_TAG_EXPORT: &str = "result_tag";
const WASM_COMPLETION_KIND_EXPORT: &str = "completion_kind";
const WASM_THROW_ERROR_NAME_EXPORT: &str = "throw_error_name";
const WASM_HOST_IMPORT_NAMESPACE: &str = "porf_host";
const WASM_HOST_IMPORT_PRINT_LINE_UTF8: &str = "print_line_utf8";
const WASM_MODULE_MEMORY_CACHE_ENTRIES: usize = 64;
/// Cut over before the multi-megabyte function bodies seen in slow Test262
/// artifacts can exhaust Cranelift's fast-compilation per-function limits.
/// This is a performance heuristic only; the normal compiler retains its
/// authoritative `CodeTooLarge` fallback below the cutoff.
const SIZE_OPTIMIZED_WASM_MIN_CODE_BODY_BYTES: usize = 1024 * 1024;
/// Large AOT functions can have multi-megabyte native stack frames even
/// without deep JavaScript recursion. Keep half of the 64MiB worker stack
/// available to Wasmtime while leaving the other half for host calls.
const WASM_MAX_STACK_SIZE: usize = 32 * 1024 * 1024;
#[cfg(test)]
const WASM_STATIC_DATA_OFFSET: usize = 4096;

/// Stack size for the worker thread that runs lowering, Wasm codegen, and
/// Wasm execution.
///
/// Wasmtime's `max_wasm_stack` config (see `run_with_wasm_aot_inner`) tells
/// the engine how much of the *host* thread's real stack a Wasm call is
/// allowed to use; Wasmtime does not provide a separate stack for sync
/// execution, so the calling native thread must already have at least that
/// much stack available. Deep IR lowering/codegen recursion has the same
/// requirement. The platform default thread stack (as small as ~2MiB for
/// `cargo test` worker threads) is not big enough, so every heavy
/// compile/codegen/run entry point below is routed through
/// `run_on_sized_stack` onto a worker thread sized the same way the test262
/// harness sizes its worker threads (see `crates/porffor-test262/src/lib.rs`),
/// so this crate is safe to call from any host thread by default.
const ENGINE_WORKER_STACK_SIZE: usize = 64 * 1024 * 1024;

/// Runs `f` on a dedicated worker thread with `ENGINE_WORKER_STACK_SIZE`
/// bytes of stack, then joins and returns its result.
///
/// Uses `thread::scope` so `f` may borrow non-`'static` data (e.g. `&str`
/// source text, `&CompilationUnit`): the scope guarantees the worker
/// finishes before this function returns.
fn run_on_sized_stack<T, F>(f: F) -> T
where
    T: Send,
    F: FnOnce() -> T + Send,
{
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(ENGINE_WORKER_STACK_SIZE)
            .spawn_scoped(scope, f)
            .expect("porffor-engine worker thread should spawn")
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
}

/// Porffor-owned Wasmtime whole-module cache. It is deliberately separate from
/// Wasmtime's global default cache so Porffor can enforce its 1GiB/70% bounds
/// without changing another Wasmtime application's artifacts.
fn wasmtime_module_cache() -> Option<wasmtime::Cache> {
    static CACHE: OnceLock<Option<wasmtime::Cache>> = OnceLock::new();
    CACHE
        .get_or_init(|| match cache::module_cache() {
            Ok(cache) => Some(cache),
            Err(err) => {
                if std::env::var_os("PORFFOR_WASM_TRACE").is_some() {
                    eprintln!(
                        "porffor wasm trace: module cache unavailable, running \
                         uncached: {err}"
                    );
                }
                None
            }
        })
        .clone()
}

fn cranelift_function_cache() -> Option<Arc<cache::FunctionCache>> {
    static CACHE: OnceLock<Option<Arc<cache::FunctionCache>>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            match cache::FunctionCache::new(
                cache::function_cache_directory(),
                cache::CACHE_LIMIT_BYTES,
            ) {
                Ok(cache) => Some(Arc::new(cache)),
                Err(err) => {
                    if std::env::var_os("PORFFOR_WASM_TRACE").is_some() {
                        eprintln!(
                            "porffor wasm trace: function cache unavailable, running uncached: \
                             {err}"
                        );
                    }
                    None
                }
            }
        })
        .clone()
}

fn program_wasm_cache() -> Option<Arc<cache::FunctionCache>> {
    static CACHE: OnceLock<Option<Arc<cache::FunctionCache>>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            match cache::FunctionCache::new(
                cache::program_cache_directory(),
                cache::HALF_CACHE_LIMIT_BYTES,
            ) {
                Ok(cache) => Some(Arc::new(cache)),
                Err(err) => {
                    if std::env::var_os("PORFFOR_WASM_TRACE").is_some() {
                        eprintln!(
                            "porffor wasm trace: program Wasm cache unavailable, running \
                             uncached: {err}"
                        );
                    }
                    None
                }
            }
        })
        .clone()
}

fn compiler_fingerprint() -> &'static str {
    env!("PORFFOR_COMPILER_FINGERPRINT")
}

fn program_wasm_cache_key(source: &str, goal: ParseGoal, options: &CompileOptions) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(compiler_fingerprint().as_bytes());
    hash.update(std::env::consts::ARCH.as_bytes());
    hash.update(match goal {
        ParseGoal::Script => b"script" as &[u8],
        ParseGoal::Module => b"module" as &[u8],
    });
    hash.update([u8::from(options.optimize)]);
    if let Some(filename) = &options.filename {
        hash.update(filename.as_bytes());
    }
    if let Some(target) = &options.target_triple {
        hash.update(target.as_bytes());
    }
    hash.update(source.as_bytes());
    hash.finalize().into()
}

fn wasm_aot_program_is_cached(source: &str, goal: ParseGoal, options: &CompileOptions) -> bool {
    // This is only a scheduling hint. The execution path still reads and
    // validates the artifact, then rebuilds it if it disappeared or is corrupt.
    let key = program_wasm_cache_key(source, goal, options);
    program_wasm_cache().is_some_and(|cache| cache.contains(&key))
}

#[doc(hidden)]
pub fn wasm_aot_script_is_cached(source: &str, options: &CompileOptions) -> bool {
    wasm_aot_program_is_cached(source, ParseGoal::Script, options)
}

#[doc(hidden)]
pub fn wasm_aot_module_is_cached(source: &str, options: &CompileOptions) -> bool {
    wasm_aot_program_is_cached(source, ParseGoal::Module, options)
}

static COMPILATION_JOBS: OnceLock<usize> = OnceLock::new();
static COMPILATION_POOL: OnceLock<Result<rayon::ThreadPool, String>> = OnceLock::new();

/// Default Cranelift parallelism: half the logical CPUs, with a minimum of one.
/// Test262's `--threads` remains a separate case-execution setting.
pub fn default_compilation_jobs() -> usize {
    std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(2)
        .div_ceil(2)
        .max(1)
}

/// Selects the shared Cranelift compilation-pool size before the first Wasm
/// module is compiled. A process has one pool so concurrent Test262 workers
/// cannot each create an unbounded set of compiler threads.
pub fn configure_compilation_jobs(jobs: usize) -> Result<(), String> {
    if jobs == 0 {
        return Err("--jobs must be a positive integer".to_string());
    }
    let configured = COMPILATION_JOBS.get_or_init(|| jobs);
    if *configured == jobs {
        Ok(())
    } else {
        Err(format!(
            "Wasm compilation pool already configured for {configured} jobs"
        ))
    }
}

pub fn compilation_jobs() -> usize {
    *COMPILATION_JOBS.get_or_init(default_compilation_jobs)
}

fn compilation_pool() -> Result<&'static rayon::ThreadPool, EngineError> {
    let result = COMPILATION_POOL.get_or_init(|| {
        let jobs = compilation_jobs();
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .thread_name(|index| format!("porffor-cranelift-{index}"))
            .build()
            .map_err(|err| format!("failed to build {jobs}-thread compilation pool: {err}"))
    });
    result.as_ref().map_err(|err| EngineError::new(err.clone()))
}

#[derive(Clone, Copy)]
enum WasmModuleMemoryCachePolicy {
    Retain,
    BypassRetention,
}

enum WasmModuleMemoryCacheOutcome {
    Hit,
    Miss,
    Bypassed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WasmNativeCompilationMode {
    Fast,
    SizeOptimized,
}

impl WasmNativeCompilationMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::SizeOptimized => "size-optimized",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WasmNativeCompilationPlan {
    mode: WasmNativeCompilationMode,
    largest_code_body_bytes: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WasmModuleMemoryCacheKey {
    native_compilation_mode: WasmNativeCompilationMode,
    wasm_sha256: [u8; 32],
}

fn memory_wasm_modules() -> &'static Mutex<VecDeque<(WasmModuleMemoryCacheKey, WasmtimeModule)>> {
    static MODULES: OnceLock<Mutex<VecDeque<(WasmModuleMemoryCacheKey, WasmtimeModule)>>> =
        OnceLock::new();
    MODULES.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn plan_wasm_native_compilation(bytes: &[u8]) -> WasmNativeCompilationPlan {
    let largest_code_body_bytes = largest_wasm_code_body_size(bytes);
    let mode = if largest_code_body_bytes
        .is_some_and(|size| size >= SIZE_OPTIMIZED_WASM_MIN_CODE_BODY_BYTES)
    {
        WasmNativeCompilationMode::SizeOptimized
    } else {
        WasmNativeCompilationMode::Fast
    };
    WasmNativeCompilationPlan {
        mode,
        largest_code_body_bytes,
    }
}

fn largest_wasm_code_body_size(bytes: &[u8]) -> Option<usize> {
    let mut largest_code_body_bytes = 0;
    for payload in WasmParser::new(0).parse_all(bytes) {
        let payload = payload.ok()?;
        if let WasmPayload::CodeSectionEntry(body) = payload {
            largest_code_body_bytes = largest_code_body_bytes.max(body.range().len());
        }
    }
    Some(largest_code_body_bytes)
}

fn wasm_module_memory_cache_key(
    bytes: &[u8],
    native_compilation_mode: WasmNativeCompilationMode,
) -> WasmModuleMemoryCacheKey {
    WasmModuleMemoryCacheKey {
        native_compilation_mode,
        wasm_sha256: Sha256::digest(bytes).into(),
    }
}

fn compile_wasm_module(
    engine: &WasmtimeEngine,
    bytes: &[u8],
) -> Result<WasmtimeModule, EngineError> {
    compilation_pool()?
        .install(|| WasmtimeModule::new(engine, bytes))
        .map_err(|err| EngineError::new(format!("wasmtime module validation failed: {err:#}")))
}

fn memory_cached_wasm_module(
    engine: &WasmtimeEngine,
    bytes: &[u8],
    native_compilation_mode: WasmNativeCompilationMode,
) -> Result<(WasmtimeModule, WasmModuleMemoryCacheOutcome), EngineError> {
    let key = wasm_module_memory_cache_key(bytes, native_compilation_mode);
    let modules = memory_wasm_modules();
    {
        let mut modules = modules
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(index) = modules.iter().position(|(candidate, _)| *candidate == key) {
            let entry = modules
                .remove(index)
                .expect("module cache index should exist");
            let module = entry.1.clone();
            modules.push_back(entry);
            return Ok((module, WasmModuleMemoryCacheOutcome::Hit));
        }
    }

    let module = compile_wasm_module(engine, bytes)?;
    let mut modules = modules
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if modules.len() == WASM_MODULE_MEMORY_CACHE_ENTRIES {
        modules.pop_front();
    }
    modules.push_back((key, module.clone()));
    Ok((module, WasmModuleMemoryCacheOutcome::Miss))
}

fn wasm_module_for_execution(
    engine: &WasmtimeEngine,
    bytes: &[u8],
    memory_cache_policy: WasmModuleMemoryCachePolicy,
    native_compilation_mode: WasmNativeCompilationMode,
) -> Result<(WasmtimeModule, WasmModuleMemoryCacheOutcome), EngineError> {
    match memory_cache_policy {
        WasmModuleMemoryCachePolicy::Retain => {
            memory_cached_wasm_module(engine, bytes, native_compilation_mode)
        }
        WasmModuleMemoryCachePolicy::BypassRetention => Ok((
            compile_wasm_module(engine, bytes)?,
            WasmModuleMemoryCacheOutcome::Bypassed,
        )),
    }
}

#[cfg(test)]
fn memory_wasm_module_is_cached(bytes: &[u8]) -> bool {
    let wasm_sha256: [u8; 32] = Sha256::digest(bytes).into();
    memory_wasm_modules()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .iter()
        .any(|(candidate, _)| candidate.wasm_sha256 == wasm_sha256)
}

/// Wall-clock tick between `Engine::increment_epoch()` calls made by
/// `ensure_wasm_epoch_ticker`. This is the granularity of Wasm-AOT execution
/// timeouts: a requested timeout is rounded up to the nearest multiple of
/// this duration. 100ms keeps the ticker thread cheap (a handful of wakeups
/// per second, process-wide, not per test) while staying far tighter than
/// the tens-of-seconds bounds test262 runs use in practice.
const WASM_EPOCH_TICK_MS: u64 = 100;

/// Generous per-store linear memory cap applied via `wasmtime::StoreLimits`.
/// This exists only to stop a pathological Wasm-AOT module that both loops
/// forever *and* keeps allocating (so epoch interruption alone would not
/// bound its memory use before the interrupt is observed) from growing
/// without bound and OOM-killing the whole in-process worker. 1GiB is far
/// above what any legitimate test262 case needs, so this must never reject a
/// conformant test.
const WASM_STORE_MEMORY_CAP_BYTES: usize = 1024 * 1024 * 1024;
const WASM_LINEAR_MEMORY_GUARD_BYTES: u64 = 32 * 1024 * 1024;

fn configure_wasm_linear_memory(config: &mut WasmtimeConfig) {
    // Shared Wasm memories cannot relocate, so their initial reservation must
    // cover every byte permitted by StoreLimits. Ordinary memories may
    // relocate, but the same cap means no valid growth can exhaust this
    // reservation.
    config.memory_reservation(WASM_STORE_MEMORY_CAP_BYTES as u64);
    config.memory_reservation_for_growth(0);
    config.memory_may_move(true);
    config.memory_guard_size(WASM_LINEAR_MEMORY_GUARD_BYTES);
    config.guard_before_linear_memory(true);
}

/// The fast-compilation `wasmtime::Engine` used for ordinary Wasm-AOT modules.
///
/// Built once rather than per run. Oversized modules use the companion
/// size-optimized engine below; both engines share immutable configuration and
/// compiled-code caches. An immutable `Module` may come from the in-memory LRU,
/// while each execution gets a fresh `Store` and `Instance`, so no JavaScript
/// state crosses runs.
fn shared_wasm_engine() -> Result<WasmtimeEngine, EngineError> {
    static ENGINE: OnceLock<Result<WasmtimeEngine, String>> = OnceLock::new();
    ENGINE
        .get_or_init(|| {
            let mut config = WasmtimeConfig::new();
            config.cranelift_opt_level(OptLevel::None);
            config.cranelift_regalloc_algorithm(RegallocAlgorithm::SinglePass);
            config.max_wasm_stack(WASM_MAX_STACK_SIZE);
            configure_wasm_linear_memory(&mut config);
            config.wasm_threads(true);
            config.wasm_function_references(true);
            config.wasm_gc(true);
            config.wasm_exceptions(true);
            config.wasm_tail_call(true);
            config.parallel_compilation(compilation_jobs() > 1);
            config.cache(wasmtime_module_cache());
            if let Some(function_cache) = cranelift_function_cache() {
                config
                    .enable_incremental_compilation(function_cache)
                    .map_err(|err| format!("Cranelift function-cache setup failed: {err}"))?;
            }
            if std::env::var_os("PORFFOR_VERIFY_FUNCTION_CACHE").is_some() {
                // This is Cranelift's own diagnostic flag: it recompiles a
                // sampled cache hit and asserts that the native result is
                // byte-for-byte identical. The flag name is intentionally
                // confined to this opt-in CI/debug path because Cranelift
                // marks stringly-typed compiler flags as an unsafe API.
                unsafe {
                    config.cranelift_flag_enable("enable_incremental_compilation_cache_checks");
                }
            }
            // Instruments emitted Wasm with epoch checks at loop back-edges
            // and function entries. Combined with `ensure_wasm_epoch_ticker`
            // and a per-store `set_epoch_deadline` below, this is what lets
            // Wasm-AOT execution run in-process by default: a hanging/looping
            // module traps out on its own instead of requiring a child
            // process to `kill()` as the only way to bound a hang.
            config.epoch_interruption(true);
            WasmtimeEngine::new(&config)
                .map_err(|err| format!("wasmtime engine setup failed: {err}"))
        })
        .clone()
        .map_err(EngineError::new)
}

fn shared_size_optimized_wasm_engine() -> Result<WasmtimeEngine, EngineError> {
    static ENGINE: OnceLock<Result<WasmtimeEngine, String>> = OnceLock::new();
    ENGINE
        .get_or_init(|| {
            let mut config = WasmtimeConfig::new();
            config.cranelift_opt_level(OptLevel::SpeedAndSize);
            config.cranelift_regalloc_algorithm(RegallocAlgorithm::SinglePass);
            config.max_wasm_stack(WASM_MAX_STACK_SIZE);
            configure_wasm_linear_memory(&mut config);
            config.wasm_threads(true);
            config.wasm_function_references(true);
            config.wasm_gc(true);
            config.wasm_exceptions(true);
            config.wasm_tail_call(true);
            config.parallel_compilation(compilation_jobs() > 1);
            config.cache(wasmtime_module_cache());
            if let Some(function_cache) = cranelift_function_cache() {
                config
                    .enable_incremental_compilation(function_cache)
                    .map_err(|err| {
                        format!("size-optimized Cranelift function-cache setup failed: {err}")
                    })?;
            }
            config.epoch_interruption(true);
            WasmtimeEngine::new(&config)
                .map_err(|err| format!("size-optimized wasmtime engine setup failed: {err}"))
        })
        .clone()
        .map_err(EngineError::new)
}

fn registered_wasm_epoch_engines() -> &'static Mutex<Vec<wasmtime::EngineWeak>> {
    static ENGINES: OnceLock<Mutex<Vec<wasmtime::EngineWeak>>> = OnceLock::new();
    ENGINES.get_or_init(|| Mutex::new(Vec::new()))
}

/// Registers an engine with the process-wide ticker that increments every
/// Wasm-AOT engine's epoch every `WASM_EPOCH_TICK_MS`. Every store created by
/// `run_with_wasm_aot_inner` sets its epoch deadline in units of this tick, so
/// both native-compilation engines must remain registered: otherwise a module
/// compiled by the engine that registered second could never time out.
fn ensure_wasm_epoch_ticker(engine: &WasmtimeEngine) {
    {
        let mut engines = registered_wasm_epoch_engines()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        engines.retain(|registered| registered.upgrade().is_some());
        let already_registered = engines.iter().any(|registered| {
            registered
                .upgrade()
                .is_some_and(|registered| WasmtimeEngine::same(&registered, engine))
        });
        if !already_registered {
            engines.push(engine.weak());
        }
    }

    static STARTED: OnceLock<()> = OnceLock::new();
    STARTED.get_or_init(|| {
        std::thread::Builder::new()
            .name("porffor-wasm-epoch-ticker".to_string())
            .spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(WASM_EPOCH_TICK_MS));
                let mut engines = registered_wasm_epoch_engines()
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                engines.retain(|registered| {
                    let Some(engine) = registered.upgrade() else {
                        return false;
                    };
                    engine.increment_epoch();
                    true
                });
            })
            .expect("porffor wasm epoch ticker thread should spawn");
    });
}

/// True if `err` (from a trapped `wasmtime` call) is specifically the
/// epoch-interruption trap raised when a store's `set_epoch_deadline` bound
/// was exceeded, as opposed to any other kind of wasm trap (unreachable,
/// stack overflow, out-of-bounds access, ...).
fn is_wasm_epoch_interrupt(err: &wasmtime::Error) -> bool {
    err.downcast_ref::<WasmtimeTrap>() == Some(&WasmtimeTrap::Interrupt)
}

pub use porffor_runtime::{
    AgentId, GlobalEnvironmentId, HostHooks, IntrinsicDescriptor, IntrinsicFunctionMetadata,
    IntrinsicId, IntrinsicKind, IntrinsicPropertyAttributes, IntrinsicPropertyDescriptor,
    IntrinsicPropertyKey, IntrinsicPropertyValue, IntrinsicRole, NullHostHooks, Realm,
    RealmBuilder, RealmGlobal, RealmId, RealmIntrinsics, RealmObjectId, RealmObjectKind,
    INTRINSIC_DESCRIPTORS, INTRINSIC_PROPERTY_DESCRIPTORS,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactKind {
    Wasm,
    C,
    Native,
}

/// The product execution backend selector.
///
/// `WasmAot` is the only backend whose results count as conformance and it is
/// the default everywhere (engine, CLI `run`/`build`/`test262`). `SpecExec`
/// wraps an interpreter (Boa) and exists solely as a hidden, developer-only
/// differential oracle for T25-style testing; per `AGENTS.md` it must never be
/// the product default, a silent fallback, or linked into product/release
/// builds. Its implementation is gated behind the `spec-exec-oracle` cargo
/// feature, which is off by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ExecutionBackend {
    SpecExec,
    #[default]
    WasmAot,
}

impl ExecutionBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            ExecutionBackend::SpecExec => "spec-exec",
            ExecutionBackend::WasmAot => "wasm-aot",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub kind: ArtifactKind,
    pub bytes: Vec<u8>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileOptions {
    pub filename: Option<String>,
    pub optimize: bool,
    pub target_triple: Option<String>,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            filename: None,
            optimize: true,
            target_triple: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunOptions {
    pub backend: ExecutionBackend,
    pub argv: Vec<String>,
    pub module_root: Option<String>,
    pub test_path: Option<String>,
    pub can_block: bool,
    /// Wall-clock bound for `ExecutionBackend::WasmAot` execution, enforced
    /// in-process via wasmtime epoch interruption (see
    /// `run_with_wasm_aot_inner`). `None` means "no caller-specified bound";
    /// execution still runs under epoch interruption but with an
    /// effectively-unbounded deadline, so it behaves as before this field
    /// existed. Ignored by other backends.
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationUnit {
    pub source: SourceUnit,
    pub ir: ProgramIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutcome {
    pub backend_used: ExecutionBackend,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectionReport {
    pub goal: ParseGoal,
    pub source_len: usize,
    pub stages: Vec<&'static str>,
    pub invariants: Vec<&'static str>,
    pub ir_summary: String,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineError {
    message: String,
    parse_diagnostic: Option<ParseDiagnostic>,
    ir_diagnostic: Option<IrDiagnostic>,
}

impl EngineError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            parse_diagnostic: None,
            ir_diagnostic: None,
        }
    }

    fn from_parse_error(err: porffor_front::ParseError) -> Self {
        Self {
            message: err.to_string(),
            parse_diagnostic: Some(err.diagnostic().clone()),
            ir_diagnostic: None,
        }
    }

    fn from_ir_diagnostic(diagnostic: IrDiagnostic) -> Self {
        Self {
            message: diagnostic.message.clone(),
            parse_diagnostic: None,
            ir_diagnostic: Some(diagnostic),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn parse_diagnostic(&self) -> Option<&ParseDiagnostic> {
        self.parse_diagnostic.as_ref()
    }

    pub fn ir_diagnostic(&self) -> Option<&IrDiagnostic> {
        self.ir_diagnostic.as_ref()
    }
}

impl core::fmt::Display for EngineError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for EngineError {}

pub struct Engine {
    realm: Realm,
}

#[derive(Clone)]
struct WasmHostState {
    realm: Realm,
    limits: WasmtimeStoreLimits,
}

impl Engine {
    pub fn new(realm: Realm) -> Self {
        Self { realm }
    }

    pub fn shell_name(&self) -> &str {
        &self.realm.shell_name
    }

    pub fn compile_script(
        &self,
        source: &str,
        options: CompileOptions,
    ) -> Result<CompilationUnit, EngineError> {
        self.compile_on_sized_stack(source, ParseGoal::Script, options)
    }

    pub fn compile_module(
        &self,
        source: &str,
        options: CompileOptions,
    ) -> Result<CompilationUnit, EngineError> {
        self.compile_on_sized_stack(source, ParseGoal::Module, options)
    }

    pub fn run_script(
        &self,
        source: &str,
        options: CompileOptions,
        run: RunOptions,
    ) -> Result<RunOutcome, EngineError> {
        if run.backend == ExecutionBackend::SpecExec {
            return self.run_with_spec_exec(
                source,
                options.filename.as_deref(),
                ParseGoal::Script,
                run,
            );
        }
        self.run_source_with_cached_wasm(source, ParseGoal::Script, options, run.timeout_ms)
    }

    pub fn run_module(
        &self,
        source: &str,
        options: CompileOptions,
        run: RunOptions,
    ) -> Result<RunOutcome, EngineError> {
        if run.backend == ExecutionBackend::SpecExec {
            return self.run_with_spec_exec(
                source,
                options.filename.as_deref(),
                ParseGoal::Module,
                run,
            );
        }
        self.run_source_with_cached_wasm(source, ParseGoal::Module, options, run.timeout_ms)
    }

    /// Runs a script through the Wasm-AOT backend on the calling thread.
    ///
    /// Callers must provide a thread with at least 64MiB of stack. This is
    /// for hosts that already own such a persistent worker; ordinary callers
    /// should use [`Engine::run_script`]. The persistent-worker path bypasses
    /// in-memory compiled-module retention so unique-source workloads remain
    /// memory-bounded; the on-disk caches remain enabled.
    #[doc(hidden)]
    pub fn run_wasm_aot_script_on_current_thread(
        &self,
        source: &str,
        options: CompileOptions,
        timeout_ms: Option<u64>,
    ) -> Result<RunOutcome, EngineError> {
        self.run_source_with_cached_wasm_on_current_thread(
            source,
            ParseGoal::Script,
            options,
            timeout_ms,
            WasmModuleMemoryCachePolicy::BypassRetention,
        )
    }

    /// Runs a module through the Wasm-AOT backend on the calling thread.
    ///
    /// Callers must provide a thread with at least 64MiB of stack. This is
    /// for hosts that already own such a persistent worker; ordinary callers
    /// should use [`Engine::run_module`]. The persistent-worker path bypasses
    /// in-memory compiled-module retention so unique-source workloads remain
    /// memory-bounded; the on-disk caches remain enabled.
    #[doc(hidden)]
    pub fn run_wasm_aot_module_on_current_thread(
        &self,
        source: &str,
        options: CompileOptions,
        timeout_ms: Option<u64>,
    ) -> Result<RunOutcome, EngineError> {
        self.run_source_with_cached_wasm_on_current_thread(
            source,
            ParseGoal::Module,
            options,
            timeout_ms,
            WasmModuleMemoryCachePolicy::BypassRetention,
        )
    }

    fn run_source_with_cached_wasm(
        &self,
        source: &str,
        goal: ParseGoal,
        options: CompileOptions,
        timeout_ms: Option<u64>,
    ) -> Result<RunOutcome, EngineError> {
        run_on_sized_stack(|| {
            self.run_source_with_cached_wasm_on_current_thread(
                source,
                goal,
                options,
                timeout_ms,
                WasmModuleMemoryCachePolicy::Retain,
            )
        })
    }

    fn run_source_with_cached_wasm_on_current_thread(
        &self,
        source: &str,
        goal: ParseGoal,
        options: CompileOptions,
        timeout_ms: Option<u64>,
        memory_cache_policy: WasmModuleMemoryCachePolicy,
    ) -> Result<RunOutcome, EngineError> {
        let key = program_wasm_cache_key(source, goal, &options);
        if let Some(cache) = program_wasm_cache() {
            let cache_started = std::time::Instant::now();
            if let Some(bytes) = cache.read(&key) {
                if std::env::var_os("PORFFOR_WASM_TRACE").is_some() {
                    eprintln!(
                        "porffor wasm trace: program-cache hit: {} bytes in {:?}",
                        bytes.len(),
                        cache_started.elapsed()
                    );
                }
                match self.run_with_wasm_bytes_inner(&bytes, timeout_ms, memory_cache_policy) {
                    Err(err)
                        if err
                            .message()
                            .starts_with("wasmtime module validation failed:") =>
                    {
                        // An incomplete/corrupted local artifact is a cache
                        // miss, never a product failure. Remove it and rebuild
                        // from the JavaScript source below.
                        cache.remove(&key);
                    }
                    result => return result,
                }
            } else if std::env::var_os("PORFFOR_WASM_TRACE").is_some() {
                eprintln!(
                    "porffor wasm trace: program-cache miss: {:?}",
                    cache_started.elapsed()
                );
            }
        }

        let unit = self.compile_on_current_thread(source, goal, options)?;
        let emit_started = std::time::Instant::now();
        let artifact = self.emit_wasm_on_current_thread(&unit)?;
        if std::env::var_os("PORFFOR_WASM_TRACE").is_some() {
            eprintln!(
                "porffor wasm trace: emit: {:?} ({} bytes)",
                emit_started.elapsed(),
                artifact.bytes.len()
            );
        }
        if let Some(cache) = program_wasm_cache() {
            let cache_started = std::time::Instant::now();
            if !cache.write(&key, artifact.bytes.clone())
                && std::env::var_os("PORFFOR_WASM_TRACE").is_some()
            {
                eprintln!("porffor wasm trace: program-cache write failed");
            } else if std::env::var_os("PORFFOR_WASM_TRACE").is_some() {
                eprintln!(
                    "porffor wasm trace: program-cache write: {:?}",
                    cache_started.elapsed()
                );
            }
        }
        self.run_with_wasm_bytes_inner(&artifact.bytes, timeout_ms, memory_cache_policy)
    }

    pub fn emit_wasm(&self, unit: &CompilationUnit) -> Result<Artifact, EngineError> {
        run_on_sized_stack(|| self.emit_wasm_on_current_thread(unit))
    }

    fn emit_wasm_on_current_thread(&self, unit: &CompilationUnit) -> Result<Artifact, EngineError> {
        match porffor_aot_wasm::emit(&unit.ir) {
            Ok(wasm) => Ok(Artifact {
                kind: ArtifactKind::Wasm,
                bytes: wasm.bytes,
                description: wasm.invariant_note.to_string(),
            }),
            Err(err) => Err(EngineError::new(format!(
                "{}. Product invariant: compile JavaScript directly to Wasm; do not ship interpreter-in-Wasm.",
                err
            ))),
        }
    }

    pub fn emit_c(&self, unit: &CompilationUnit) -> Result<Artifact, EngineError> {
        run_on_sized_stack(|| match porffor_backend_c::emit(&unit.ir) {
            Ok(c) => Ok(Artifact {
                kind: ArtifactKind::C,
                bytes: c.source.into_bytes(),
                description: "shared IR to C artifact".to_string(),
            }),
            Err(err) => Err(EngineError::new(err)),
        })
    }

    pub fn emit_native(
        &self,
        unit: &CompilationUnit,
        target_triple: Option<&str>,
    ) -> Result<Artifact, EngineError> {
        run_on_sized_stack(
            || match porffor_backend_native::emit(&unit.ir, target_triple) {
                Ok(native) => Ok(Artifact {
                    kind: ArtifactKind::Native,
                    bytes: Vec::new(),
                    description: format!(
                        "native artifact placeholder for {:?}",
                        native.target_triple
                    ),
                }),
                Err(err) => Err(EngineError::new(err)),
            },
        )
    }

    pub fn inspect(&self, unit: &CompilationUnit) -> InspectionReport {
        InspectionReport {
            goal: unit.source.goal,
            source_len: unit.ir.source_len,
            stages: unit
                .ir
                .stages
                .iter()
                .map(|stage| match stage {
                    porffor_ir::LoweringStage::ParsedSource => "parsed-source",
                    porffor_ir::LoweringStage::AstReparsed => "ast-reparsed",
                    porffor_ir::LoweringStage::ScriptIrBuilt => "script-ir-built",
                    porffor_ir::LoweringStage::UnsupportedFeaturesRecorded => {
                        "unsupported-features-recorded"
                    }
                    porffor_ir::LoweringStage::WasmReady => "wasm-ready",
                })
                .collect(),
            invariants: unit.ir.invariants.clone(),
            ir_summary: unit.ir.ir_summary(),
            diagnostics: unit
                .ir
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.clone())
                .collect(),
        }
    }

    fn compile_on_sized_stack(
        &self,
        source: &str,
        goal: ParseGoal,
        options: CompileOptions,
    ) -> Result<CompilationUnit, EngineError> {
        run_on_sized_stack(move || self.compile_on_current_thread(source, goal, options))
    }

    fn compile_on_current_thread(
        &self,
        source: &str,
        goal: ParseGoal,
        options: CompileOptions,
    ) -> Result<CompilationUnit, EngineError> {
        let trace = std::env::var_os("PORFFOR_WASM_TRACE").is_some();
        let parse_started = std::time::Instant::now();
        let source = parse(
            source,
            ParseOptions {
                goal,
                filename: options.filename,
            },
        )
        .map_err(EngineError::from_parse_error)?;
        if trace {
            eprintln!("porffor wasm trace: parse: {:?}", parse_started.elapsed());
        }
        let lower_started = std::time::Instant::now();
        let ir = lower(&source);
        if trace {
            eprintln!("porffor wasm trace: lower: {:?}", lower_started.elapsed());
        }
        if let Some(diagnostic) = ir
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.kind == IrDiagnosticKind::EarlyError)
        {
            return Err(EngineError::from_ir_diagnostic(diagnostic.clone()));
        }
        Ok(CompilationUnit { source, ir })
    }

    pub fn run_compiled_unit(
        &self,
        unit: &CompilationUnit,
        source: &str,
        run: RunOptions,
    ) -> Result<RunOutcome, EngineError> {
        match run.backend {
            ExecutionBackend::SpecExec => self.run_with_spec_exec(
                source,
                unit.source.filename.as_deref(),
                unit.source.goal,
                run,
            ),
            ExecutionBackend::WasmAot => self.run_with_wasm_aot(unit, run.timeout_ms),
        }
    }

    /// Developer-only differential oracle path (Boa interpreter). Never the
    /// product/default execution path — see `ExecutionBackend` docs. Real
    /// implementation only exists when the `spec-exec-oracle` cargo feature
    /// is enabled, keeping `boa_engine` out of product/release builds.
    #[cfg(feature = "spec-exec-oracle")]
    fn run_with_spec_exec(
        &self,
        source: &str,
        filename: Option<&str>,
        goal: ParseGoal,
        run: RunOptions,
    ) -> Result<RunOutcome, EngineError> {
        run_on_sized_stack(move || {
            let outcome = match goal {
                ParseGoal::Module => porffor_spec_exec::execute_module(
                    source,
                    filename,
                    porffor_spec_exec::ModuleHostConfig {
                        module_root: run.module_root.clone().map(Into::into),
                        test_path: run.test_path.clone().map(Into::into),
                    },
                    &run.argv,
                    run.can_block,
                ),
                ParseGoal::Script => {
                    porffor_spec_exec::execute_script(source, filename, &run.argv, run.can_block)
                }
            }
            .map_err(|err| EngineError::new(err.to_string()))?;

            Ok(RunOutcome {
                backend_used: ExecutionBackend::SpecExec,
                note: outcome.note,
            })
        })
    }

    #[cfg(not(feature = "spec-exec-oracle"))]
    fn run_with_spec_exec(
        &self,
        _source: &str,
        _filename: Option<&str>,
        _goal: ParseGoal,
        _run: RunOptions,
    ) -> Result<RunOutcome, EngineError> {
        Err(EngineError::new(
            "spec-exec is a developer-only differential oracle backend and is not linked into \
             this build; rebuild with `--features spec-exec-oracle` (porffor-engine/porffor-cli) \
             to use it for differential testing only, never as the product execution backend",
        ))
    }

    fn run_with_wasm_aot(
        &self,
        unit: &CompilationUnit,
        timeout_ms: Option<u64>,
    ) -> Result<RunOutcome, EngineError> {
        run_on_sized_stack(|| self.run_with_wasm_aot_inner(unit, timeout_ms))
    }

    fn run_with_wasm_aot_inner(
        &self,
        unit: &CompilationUnit,
        timeout_ms: Option<u64>,
    ) -> Result<RunOutcome, EngineError> {
        let trace_wasm = std::env::var_os("PORFFOR_WASM_TRACE").is_some();
        let trace_start = std::time::Instant::now();
        let trace_phase = |phase: &str, started: std::time::Instant| {
            if trace_wasm {
                eprintln!(
                    "porffor wasm trace: {phase}: {:?} (total {:?})",
                    started.elapsed(),
                    trace_start.elapsed(),
                );
            }
        };

        let emit_started = std::time::Instant::now();
        let artifact = porffor_aot_wasm::emit(&unit.ir).map_err(|err| {
            EngineError::new(format!(
                "{}. Product invariant: compile JavaScript directly to Wasm; do not ship interpreter-in-Wasm.",
                err
            ))
        })?;
        if std::env::var_os("PORFFOR_WASM_TRACE_DUMP").is_some() {
            eprintln!(
                "porffor wasm trace: artifact debug:\n{}",
                artifact.debug_dump
            );
        }
        trace_phase("emit", emit_started);
        self.run_with_wasm_bytes_inner(
            &artifact.bytes,
            timeout_ms,
            WasmModuleMemoryCachePolicy::Retain,
        )
    }

    fn run_with_wasm_bytes_inner(
        &self,
        bytes: &[u8],
        timeout_ms: Option<u64>,
        memory_cache_policy: WasmModuleMemoryCachePolicy,
    ) -> Result<RunOutcome, EngineError> {
        let trace_wasm = std::env::var_os("PORFFOR_WASM_TRACE").is_some();
        let trace_start = std::time::Instant::now();
        let trace_phase = |phase: &str, started: std::time::Instant| {
            if trace_wasm {
                eprintln!(
                    "porffor wasm trace: {phase}: {:?} (total {:?})",
                    started.elapsed(),
                    trace_start.elapsed(),
                );
            }
        };
        if trace_wasm {
            eprintln!("porffor wasm trace: artifact bytes: {}", bytes.len());
        }

        let native_compilation_plan = plan_wasm_native_compilation(bytes);
        if trace_wasm {
            match native_compilation_plan.largest_code_body_bytes {
                Some(largest_code_body_bytes) => eprintln!(
                    "porffor wasm trace: native compiler: {} (largest code body: {} bytes, size-optimized threshold: {} bytes)",
                    native_compilation_plan.mode.as_str(),
                    largest_code_body_bytes,
                    SIZE_OPTIMIZED_WASM_MIN_CODE_BODY_BYTES
                ),
                None => eprintln!(
                    "porffor wasm trace: native compiler: fast (code-body scan could not parse artifact; Wasmtime validation remains authoritative)"
                ),
            }
        }
        let engine_started = std::time::Instant::now();
        let mut engine = match native_compilation_plan.mode {
            WasmNativeCompilationMode::Fast => shared_wasm_engine()?,
            WasmNativeCompilationMode::SizeOptimized => shared_size_optimized_wasm_engine()?,
        };
        trace_phase("engine", engine_started);
        ensure_wasm_epoch_ticker(&engine);
        let module_started = std::time::Instant::now();
        let function_cache_before = trace_wasm
            .then(cranelift_function_cache)
            .flatten()
            .map(|cache| cache.counters());
        let module_cache_before =
            wasmtime_module_cache().map(|cache| (cache.cache_hits(), cache.cache_misses()));
        let (module, memory_cache_outcome) = match wasm_module_for_execution(
            &engine,
            bytes,
            memory_cache_policy,
            native_compilation_plan.mode,
        ) {
            Ok(compiled) => compiled,
            Err(error)
                if native_compilation_plan.mode == WasmNativeCompilationMode::Fast
                    && error.to_string().contains("Code for function is too large") =>
            {
                if trace_wasm {
                    eprintln!(
                        "porffor wasm trace: native compiler fallback: size-optimized after fast compiler exceeded its per-function limit"
                    );
                }
                engine = shared_size_optimized_wasm_engine()?;
                ensure_wasm_epoch_ticker(&engine);
                (
                    compile_wasm_module(&engine, bytes)?,
                    WasmModuleMemoryCacheOutcome::Bypassed,
                )
            }
            Err(error) => return Err(error),
        };
        let module_elapsed = module_started.elapsed();
        if trace_wasm {
            eprintln!(
                "porffor wasm trace: module-memory-cache {}: {:?}",
                match memory_cache_outcome {
                    WasmModuleMemoryCacheOutcome::Hit => "hit",
                    WasmModuleMemoryCacheOutcome::Miss => "miss",
                    WasmModuleMemoryCacheOutcome::Bypassed => "bypass",
                },
                module_elapsed
            );
            if let Some(before) = function_cache_before {
                if let Some(after) = cranelift_function_cache().map(|cache| cache.counters()) {
                    eprintln!(
                        "porffor wasm trace: function-cache hits={} misses={} during {:?}",
                        after.0.saturating_sub(before.0),
                        after.1.saturating_sub(before.1),
                        module_elapsed
                    );
                }
            }
            let module_cache_after =
                wasmtime_module_cache().map(|cache| (cache.cache_hits(), cache.cache_misses()));
            match (module_cache_before, module_cache_after) {
                (Some(before), Some(after)) if after.0 > before.0 => {
                    eprintln!("porffor wasm trace: module-cache hit: {:?}", module_elapsed)
                }
                (Some(before), Some(after)) if after.1 > before.1 => {
                    eprintln!(
                        "porffor wasm trace: module-cache miss: {:?}",
                        module_elapsed
                    );
                    eprintln!(
                        "porffor wasm trace: native compilation: {:?}",
                        module_elapsed
                    );
                }
                _ => eprintln!(
                    "porffor wasm trace: module-cache result unavailable: {:?}",
                    module_elapsed
                ),
            }
        }
        trace_phase("module", module_started);
        let store_started = std::time::Instant::now();
        let mut store = WasmtimeStore::new(
            &engine,
            WasmHostState {
                realm: self.realm.clone(),
                limits: WasmtimeStoreLimitsBuilder::new()
                    .memory_size(WASM_STORE_MEMORY_CAP_BYTES)
                    .build(),
            },
        );
        store.limiter(|state| &mut state.limits);
        // The engine has epoch interruption enabled process-wide (see
        // `shared_wasm_engine`); every store must set an explicit deadline or
        // it traps immediately (deadline defaults to epoch 0, which has
        // already "elapsed"). Round the caller's timeout up to whole
        // epoch-ticker ticks; with no caller-specified timeout, set a
        // deadline so far in the future it is never practically reached, so
        // behavior matches "no timeout" rather than "immediate trap".
        let epoch_deadline_ticks = match timeout_ms {
            Some(timeout_ms) => timeout_ms.div_ceil(WASM_EPOCH_TICK_MS).max(1),
            None => u64::MAX / 2,
        };
        store.set_epoch_deadline(epoch_deadline_ticks);
        let mut linker = WasmtimeLinker::new(&engine);
        linker
            .func_wrap(
                WASM_HOST_IMPORT_NAMESPACE,
                WASM_HOST_IMPORT_PRINT_LINE_UTF8,
                |mut caller: WasmtimeCaller<'_, WasmHostState>,
                 ptr: i32,
                 len: i32|
                 -> wasmtime::Result<()> {
                    let memory = caller.get_export("memory").ok_or_else(|| {
                        wasmtime::Error::msg("wasmtime host import failed: missing exported memory")
                    })?;
                    let ptr = usize::try_from(ptr).map_err(|_| {
                        wasmtime::Error::msg("wasmtime host import failed: negative utf-8 pointer")
                    })?;
                    let len = usize::try_from(len).map_err(|_| {
                        wasmtime::Error::msg("wasmtime host import failed: negative utf-8 length")
                    })?;
                    let mut bytes = vec![0; len];
                    match memory {
                        WasmtimeExtern::Memory(memory) => {
                            memory.read(&caller, ptr, &mut bytes).map_err(|err| {
                                wasmtime::Error::msg(format!(
                                    "wasmtime host import failed: unable to read memory: {err}"
                                ))
                            })?;
                        }
                        WasmtimeExtern::SharedMemory(memory) => {
                            read_wasmtime_shared_memory(&memory, ptr, &mut bytes).map_err(
                                |err| {
                                    wasmtime::Error::msg(format!(
                                        "wasmtime host import failed: unable to read memory: {err}"
                                    ))
                                },
                            )?;
                        }
                        _ => {
                            return Err(wasmtime::Error::msg(
                                "wasmtime host import failed: missing exported memory",
                            ));
                        }
                    }
                    let text = String::from_utf8(bytes).map_err(|err| {
                        wasmtime::Error::msg(format!(
                            "wasmtime host import failed: invalid utf-8: {err}"
                        ))
                    })?;
                    caller.data().realm.host_hooks().print_line(&text);
                    Ok(())
                },
            )
            .map_err(|err| EngineError::new(format!("wasmtime linker setup failed: {err}")))?;
        trace_phase("store + linker", store_started);
        let instantiate_started = std::time::Instant::now();
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|err| EngineError::new(format!("wasmtime instantiate failed: {err}")))?;
        trace_phase("instantiate", instantiate_started);
        let lookup_started = std::time::Instant::now();
        let main = instance
            .get_typed_func::<(), i64>(&mut store, "main")
            .map_err(|err| EngineError::new(format!("wasmtime export lookup failed: {err}")))?;
        trace_phase("lookup main", lookup_started);
        let execution_started = std::time::Instant::now();
        let payload = main.call(&mut store, ()).map_err(|err| {
            if is_wasm_epoch_interrupt(&err) {
                // Distinguish this from other traps with the same "timeout
                // exceeded" phrasing the child-process/elapsed-time timeout
                // path already uses (see
                // `porffor_test262::run_one_case_in_child_process` and
                // `run_one_case`), so both paths classify identically as
                // timeouts downstream (FailureKind::Runtime,
                // OutcomeKind timeout bucketing, `RunSummary::timeouts`).
                EngineError::new(format!(
                    "timeout exceeded after {}ms (wasm epoch interrupt, bound {}ms)",
                    trace_start.elapsed().as_millis(),
                    timeout_ms.unwrap_or(0)
                ))
            } else {
                EngineError::new(format!("wasmtime execution trapped: {err:?}"))
            }
        })?;
        trace_phase("execution", execution_started);
        let result_tag = instance
            .get_global(&mut store, WASM_RESULT_TAG_EXPORT)
            .ok_or_else(|| EngineError::new("wasmtime export lookup failed: missing result_tag"))?
            .get(&mut store);
        let WasmtimeVal::I32(result_tag) = result_tag else {
            return Err(EngineError::new(
                "wasm result_tag export had unexpected type",
            ));
        };
        let result_tag = WasmRuntimeValueTag::from_tag(result_tag)
            .ok_or_else(|| EngineError::new(format!("unknown wasm result tag: {result_tag}")))?;
        let result_kind = result_tag.value_kind();
        let completion = instance
            .get_global(&mut store, WASM_COMPLETION_KIND_EXPORT)
            .ok_or_else(|| {
                EngineError::new("wasmtime export lookup failed: missing completion_kind")
            })?
            .get(&mut store);
        let WasmtimeVal::I32(completion_kind) = completion else {
            return Err(EngineError::new(
                "wasm completion_kind export had unexpected type",
            ));
        };
        let note = render_wasmtime_completion(
            result_tag,
            payload,
            wasmtime_exported_memory(&instance, &mut store),
            &mut store,
        )?;
        if completion_kind != 0 {
            let error_name = if matches!(
                result_kind,
                ValueKind::Object | ValueKind::Array | ValueKind::Function | ValueKind::Arguments
            ) {
                let memory = wasmtime_exported_memory(&instance, &mut store);
                read_wasmtime_string_payload_global(
                    &instance,
                    &mut store,
                    WASM_THROW_ERROR_NAME_EXPORT,
                    memory,
                )?
            } else {
                None
            };
            let prefix = error_name
                .filter(|name| !name.is_empty())
                .map(|name| format!("{name}: "))
                .unwrap_or_default();
            return Err(EngineError::new(format!("uncaught throw: {prefix}{note}")));
        }

        Ok(RunOutcome {
            backend_used: ExecutionBackend::WasmAot,
            note,
        })
    }
}

enum WasmtimeExportedMemory {
    Unshared(wasmtime::Memory),
    Shared(wasmtime::SharedMemory),
}

impl WasmtimeExportedMemory {
    fn read(
        &self,
        store: &mut WasmtimeStore<WasmHostState>,
        offset: usize,
        bytes: &mut [u8],
    ) -> Result<(), EngineError> {
        match self {
            Self::Unshared(memory) => memory
                .read(store, offset, bytes)
                .map_err(|err| EngineError::new(format!("failed to read wasm memory: {err}"))),
            Self::Shared(memory) => read_wasmtime_shared_memory(memory, offset, bytes),
        }
    }
}

fn wasmtime_exported_memory(
    instance: &wasmtime::Instance,
    store: &mut WasmtimeStore<WasmHostState>,
) -> Option<WasmtimeExportedMemory> {
    match instance.get_export(store, "memory")? {
        WasmtimeExtern::Memory(memory) => Some(WasmtimeExportedMemory::Unshared(memory)),
        WasmtimeExtern::SharedMemory(memory) => Some(WasmtimeExportedMemory::Shared(memory)),
        _ => None,
    }
}

fn read_wasmtime_shared_memory(
    memory: &wasmtime::SharedMemory,
    offset: usize,
    bytes: &mut [u8],
) -> Result<(), EngineError> {
    let end = offset
        .checked_add(bytes.len())
        .filter(|end| *end <= memory.data_size())
        .ok_or_else(|| EngineError::new("failed to read wasm memory: out of bounds"))?;
    for (source, destination) in memory.data()[offset..end].iter().zip(bytes) {
        // Wasmtime exposes shared bytes as UnsafeCell and requires host reads
        // to use atomics because another Wasm thread may modify them.
        *destination = unsafe { AtomicU8::from_ptr(source.get()) }.load(Ordering::Relaxed);
    }
    Ok(())
}

fn read_wasmtime_string_payload_global(
    instance: &wasmtime::Instance,
    store: &mut WasmtimeStore<WasmHostState>,
    global_name: &str,
    memory: Option<WasmtimeExportedMemory>,
) -> Result<Option<String>, EngineError> {
    let Some(global) = instance.get_global(&mut *store, global_name) else {
        return Ok(None);
    };
    let WasmtimeVal::I64(payload) = global.get(&mut *store) else {
        return Err(EngineError::new(format!(
            "wasm {global_name} export had unexpected type"
        )));
    };
    if payload == 0 {
        return Ok(None);
    }
    let memory = memory.ok_or_else(|| {
        EngineError::new(format!(
            "wasm {global_name} string needs exported memory, but none exists"
        ))
    })?;
    let offset = ((payload as u64) >> 32) as usize;
    let len = ((payload as u64) & 0xFFFF_FFFF) as usize;
    let mut bytes = vec![0; len];
    memory.read(store, offset, &mut bytes)?;
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|err| EngineError::new(format!("wasm string result is not utf-8: {err}")))
}

fn render_wasmtime_completion(
    tag: WasmRuntimeValueTag,
    payload: i64,
    memory: Option<WasmtimeExportedMemory>,
    store: &mut WasmtimeStore<WasmHostState>,
) -> Result<String, EngineError> {
    let (kind, rendered) = match tag {
        WasmRuntimeValueTag::HeapBigInt => {
            let memory = memory.ok_or_else(|| {
                EngineError::new("wasm heap BigInt result needs exported memory, but none exists")
            })?;
            let memory_byte_len = match &memory {
                WasmtimeExportedMemory::Unshared(memory) => memory.data_size(&*store),
                WasmtimeExportedMemory::Shared(memory) => memory.data_size(),
            };
            let decimal =
                decode_heap_bigint_decimal(payload as u64, memory_byte_len, |offset, bytes| {
                    memory.read(store, offset, bytes)
                })
                .map_err(|error| {
                    EngineError::new(format!(
                        "failed to decode wasm heap BigInt completion: {error}"
                    ))
                })?;
            (ValueKind::BigInt, format!("{decimal}n"))
        }
        WasmRuntimeValueTag::ValueKind(kind) => {
            let rendered = match kind {
                ValueKind::Undefined => "undefined".to_string(),
                ValueKind::Null => "null".to_string(),
                ValueKind::Boolean => {
                    if payload == 0 {
                        "false".to_string()
                    } else {
                        "true".to_string()
                    }
                }
                ValueKind::Number => format!("{}", f64::from_bits(payload as u64)),
                ValueKind::String => {
                    let offset = ((payload as u64) >> 32) as usize;
                    let len = ((payload as u64) & 0xFFFF_FFFF) as usize;
                    let memory = memory.ok_or_else(|| {
                        EngineError::new(
                            "wasm string result needs exported memory, but none exists",
                        )
                    })?;
                    let mut bytes = vec![0; len];
                    memory.read(store, offset, &mut bytes)?;
                    String::from_utf8(bytes).map_err(|err| {
                        EngineError::new(format!("wasm string result is not utf-8: {err}"))
                    })?
                }
                ValueKind::Object => format!("handle@{}", payload as u64),
                ValueKind::Array => format!("handle@{}", payload as u64),
                ValueKind::Function => format!("handle@{}", payload as u64),
                ValueKind::Arguments => format!("handle@{}", payload as u64),
                ValueKind::Symbol => format!("symbol@{}", payload as u64),
                ValueKind::BigInt => format!("{}n", payload),
                ValueKind::Dynamic => {
                    return Err(EngineError::new(
                        "wasm completion used dynamic tag; expected concrete runtime tag",
                    ));
                }
            };
            (kind, rendered)
        }
    };
    Ok(format!(
        "wasm-aot completion: {}({rendered})",
        kind.as_str()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_cache_key_tracks_source_goal_and_configuration() {
        let base = CompileOptions {
            filename: Some("case.js".to_string()),
            ..CompileOptions::default()
        };
        let key = program_wasm_cache_key("1 + 2", ParseGoal::Script, &base);
        assert_eq!(
            key,
            program_wasm_cache_key("1 + 2", ParseGoal::Script, &base)
        );
        assert_ne!(
            key,
            program_wasm_cache_key("1 + 3", ParseGoal::Script, &base)
        );
        assert_ne!(
            key,
            program_wasm_cache_key("1 + 2", ParseGoal::Module, &base)
        );
        let mut changed = base.clone();
        changed.optimize = !changed.optimize;
        assert_ne!(
            key,
            program_wasm_cache_key("1 + 2", ParseGoal::Script, &changed)
        );
    }

    fn push_unsigned_leb128(output: &mut Vec<u8>, mut value: usize) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            output.push(byte);
            if value == 0 {
                return;
            }
        }
    }

    fn append_wasm_section(module: &mut Vec<u8>, section_id: u8, payload: &[u8]) {
        module.push(section_id);
        push_unsigned_leb128(module, payload.len());
        module.extend_from_slice(payload);
    }

    fn wasm_module_with_code_body_size(code_body_size: usize) -> Vec<u8> {
        assert!(code_body_size >= 2);
        let mut module = b"\0asm\x01\0\0\0".to_vec();
        append_wasm_section(&mut module, 1, &[1, 0x60, 0, 0]);
        append_wasm_section(&mut module, 3, &[1, 0]);

        let mut code = Vec::with_capacity(code_body_size + 8);
        code.push(1);
        push_unsigned_leb128(&mut code, code_body_size);
        code.push(0);
        code.extend(std::iter::repeat_n(0x01, code_body_size - 2));
        code.push(0x0b);
        append_wasm_section(&mut module, 10, &code);
        module
    }

    #[test]
    fn oversized_wasm_code_bodies_select_size_optimized_native_compilation() {
        let below_threshold =
            wasm_module_with_code_body_size(SIZE_OPTIMIZED_WASM_MIN_CODE_BODY_BYTES - 1);
        assert_eq!(
            plan_wasm_native_compilation(&below_threshold),
            WasmNativeCompilationPlan {
                mode: WasmNativeCompilationMode::Fast,
                largest_code_body_bytes: Some(SIZE_OPTIMIZED_WASM_MIN_CODE_BODY_BYTES - 1),
            }
        );

        let at_threshold = wasm_module_with_code_body_size(SIZE_OPTIMIZED_WASM_MIN_CODE_BODY_BYTES);
        assert_eq!(
            plan_wasm_native_compilation(&at_threshold),
            WasmNativeCompilationPlan {
                mode: WasmNativeCompilationMode::SizeOptimized,
                largest_code_body_bytes: Some(SIZE_OPTIMIZED_WASM_MIN_CODE_BODY_BYTES),
            }
        );
    }

    #[test]
    fn malformed_wasm_remains_on_the_authoritative_validation_path() {
        let mut malformed = wasm_module_with_code_body_size(2);
        malformed.pop();

        assert_eq!(
            plan_wasm_native_compilation(&malformed),
            WasmNativeCompilationPlan {
                mode: WasmNativeCompilationMode::Fast,
                largest_code_body_bytes: None,
            }
        );
        let engine = shared_wasm_engine().expect("fast Wasmtime engine should initialize");
        assert!(
            WasmtimeModule::new(&engine, &malformed).is_err(),
            "malformed Wasm must still be rejected by Wasmtime"
        );
    }

    #[test]
    fn module_memory_cache_separates_native_compilation_modes() {
        let wasm = wasm_module_with_code_body_size(2);
        let fast_key = wasm_module_memory_cache_key(&wasm, WasmNativeCompilationMode::Fast);
        let size_optimized_key =
            wasm_module_memory_cache_key(&wasm, WasmNativeCompilationMode::SizeOptimized);

        assert_eq!(fast_key.wasm_sha256, size_optimized_key.wasm_sha256);
        assert_ne!(fast_key, size_optimized_key);
    }

    #[cfg(all(target_os = "linux", target_pointer_width = "64"))]
    fn process_virtual_memory_bytes() -> u64 {
        let status = std::fs::read_to_string("/proc/self/status")
            .expect("Linux process status should exist");
        let vm_size = status
            .lines()
            .find_map(|line| line.strip_prefix("VmSize:"))
            .expect("Linux process status should report VmSize");
        let mut fields = vm_size.split_ascii_whitespace();
        let kibibytes = fields
            .next()
            .expect("VmSize should contain a number")
            .parse::<u64>()
            .expect("VmSize should be numeric");
        assert_eq!(
            fields.next(),
            Some("kB"),
            "VmSize should be reported in kibibytes: {vm_size}"
        );
        kibibytes
            .checked_mul(1024)
            .expect("VmSize should fit in bytes")
    }

    #[cfg(all(target_os = "linux", target_pointer_width = "64"))]
    #[test]
    fn three_live_wasm_stores_reserve_less_than_four_gibibytes() {
        const EXPORTED_MEMORY_WASM: &[u8] = &[
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07,
            0x0a, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00,
        ];
        const MAXIMUM_ADDITIONAL_VIRTUAL_MEMORY_BYTES: u64 = 4 * 1024 * 1024 * 1024;

        let engines = [
            (
                "fast",
                shared_wasm_engine().expect("fast Wasmtime engine should initialize"),
            ),
            (
                "size-optimized",
                shared_size_optimized_wasm_engine()
                    .expect("size-optimized Wasmtime engine should initialize"),
            ),
        ];
        for (engine_name, engine) in engines {
            let module = WasmtimeModule::new(&engine, EXPORTED_MEMORY_WASM)
                .expect("ordinary memory probe should compile");
            let virtual_memory_before = process_virtual_memory_bytes();
            let mut live_instances = Vec::with_capacity(3);
            for store_index in 0..3 {
                let mut store = WasmtimeStore::new(&engine, ());
                let instance =
                    wasmtime::Instance::new(&mut store, &module, &[]).unwrap_or_else(|error| {
                        panic!(
                            "{engine_name} Wasmtime store {store_index} should instantiate: {error}"
                        )
                    });
                live_instances.push((store, instance));
            }
            std::hint::black_box(&live_instances);
            let additional_virtual_memory =
                process_virtual_memory_bytes().saturating_sub(virtual_memory_before);

            assert!(
                additional_virtual_memory < MAXIMUM_ADDITIONAL_VIRTUAL_MEMORY_BYTES,
                "{engine_name} Wasmtime engine reserved {additional_virtual_memory} bytes for \
                 three live ordinary memories; expected less than \
                 {MAXIMUM_ADDITIONAL_VIRTUAL_MEMORY_BYTES}"
            );
        }
    }

    #[test]
    fn shared_memory_reaches_but_cannot_exceed_the_one_gibibyte_cap() {
        const EXPORTED_SHARED_MEMORY_WASM: &[u8] = &[
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x05, 0x06, 0x01, 0x03, 0x01, 0x80,
            0x80, 0x04, 0x07, 0x0a, 0x01, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00,
        ];
        const WASM_PAGE_BYTES: usize = 64 * 1024;
        const ONE_GIBIBYTE_PAGES: u64 = 16_384;

        assert_eq!(
            WASM_STORE_MEMORY_CAP_BYTES / WASM_PAGE_BYTES,
            ONE_GIBIBYTE_PAGES as usize,
            "the Wasm store memory cap must remain one GiB"
        );
        let engines = [
            (
                "fast",
                shared_wasm_engine().expect("fast Wasmtime engine should initialize"),
            ),
            (
                "size-optimized",
                shared_size_optimized_wasm_engine()
                    .expect("size-optimized Wasmtime engine should initialize"),
            ),
        ];
        for (engine_name, engine) in engines {
            let module = WasmtimeModule::new(&engine, EXPORTED_SHARED_MEMORY_WASM)
                .expect("shared memory probe should compile");
            let limits = WasmtimeStoreLimitsBuilder::new()
                .memory_size(WASM_STORE_MEMORY_CAP_BYTES)
                .build();
            let mut store = WasmtimeStore::new(&engine, limits);
            store.limiter(|limits| limits);
            let instance =
                wasmtime::Instance::new(&mut store, &module, &[]).unwrap_or_else(|error| {
                    panic!("{engine_name} shared memory should instantiate: {error}")
                });
            let memory = instance
                .get_shared_memory(&mut store, "memory")
                .expect("shared memory probe should export memory");

            assert_eq!(memory.size(), 1);
            assert_eq!(
                memory.grow(ONE_GIBIBYTE_PAGES - 1).unwrap_or_else(|error| {
                    panic!("{engine_name} shared memory should grow to one GiB: {error}")
                }),
                1
            );
            assert_eq!(memory.size(), ONE_GIBIBYTE_PAGES);
            assert!(
                memory.grow(1).is_err(),
                "{engine_name} shared memory must not grow beyond one GiB"
            );
        }
    }

    fn assert_epoch_ticker_advances(engine: &WasmtimeEngine) {
        const EXPORTED_NOOP_WASM: &[u8] = &[
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x74, 0x69, 0x63, 0x6b, 0x00, 0x00,
            0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        ensure_wasm_epoch_ticker(engine);
        let module = WasmtimeModule::new(engine, EXPORTED_NOOP_WASM)
            .expect("epoch probe module should compile");
        let mut store = WasmtimeStore::new(engine, ());
        let instance = wasmtime::Instance::new(&mut store, &module, &[])
            .expect("epoch probe module should instantiate");
        let tick = instance
            .get_typed_func::<(), ()>(&mut store, "tick")
            .expect("epoch probe function should exist");
        store.set_epoch_deadline(1);

        let timeout = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            match tick.call(&mut store, ()) {
                Err(error) => {
                    assert!(
                        is_wasm_epoch_interrupt(&error),
                        "epoch probe should stop with an interrupt trap: {error}"
                    );
                    return;
                }
                Ok(()) if std::time::Instant::now() < timeout => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Ok(()) => panic!("registered Wasmtime engine did not receive an epoch tick"),
            }
        }
    }

    #[test]
    fn epoch_ticker_advances_both_native_compilation_engines() {
        let fast = shared_wasm_engine().expect("fast Wasmtime engine should initialize");
        assert_epoch_ticker_advances(&fast);

        let size_optimized = shared_size_optimized_wasm_engine()
            .expect("size-optimized Wasmtime engine should initialize");
        assert_epoch_ticker_advances(&size_optimized);
    }

    use std::sync::{Arc, Mutex};
    use wasmparser::{Imports, Parser, Payload};

    #[derive(Debug)]
    struct CapturingHostHooks {
        lines: Arc<Mutex<Vec<String>>>,
    }

    impl HostHooks for CapturingHostHooks {
        fn print_line(&self, text: &str) {
            self.lines
                .lock()
                .expect("capture mutex poisoned")
                .push(text.to_string());
        }
    }

    fn engine() -> Engine {
        Engine::new(RealmBuilder::new().build())
    }

    fn engine_with_captured_prints(lines: Arc<Mutex<Vec<String>>>) -> Engine {
        Engine::new(
            RealmBuilder::new()
                .with_host_hooks(Box::new(CapturingHostHooks { lines }))
                .build(),
        )
    }

    fn run_wasm_raw(
        source: &str,
    ) -> (
        i64,
        WasmRuntimeValueTag,
        i32,
        Option<Vec<u8>>,
        Option<Vec<u8>>,
        Option<Vec<u8>>,
    ) {
        let engine = engine();
        let unit = engine
            .compile_script(source, CompileOptions::default())
            .expect("script compile should succeed");
        let artifact = engine.emit_wasm(&unit).expect("wasm emit should succeed");
        let wasm_engine = shared_wasm_engine().expect("wasmtime engine should initialize");
        let module = WasmtimeModule::new(&wasm_engine, &artifact.bytes)
            .expect("module should validate for the supported Wasmtime target");
        let mut store = WasmtimeStore::new(
            &wasm_engine,
            WasmHostState {
                realm: engine.realm.clone(),
                limits: WasmtimeStoreLimitsBuilder::new()
                    .memory_size(WASM_STORE_MEMORY_CAP_BYTES)
                    .build(),
            },
        );
        store.limiter(|state| &mut state.limits);
        store.set_epoch_deadline(u64::MAX / 2);
        let mut linker = WasmtimeLinker::new(&wasm_engine);
        linker
            .func_wrap(
                WASM_HOST_IMPORT_NAMESPACE,
                WASM_HOST_IMPORT_PRINT_LINE_UTF8,
                |_caller: WasmtimeCaller<'_, WasmHostState>,
                 _ptr: i32,
                 _len: i32|
                 -> wasmtime::Result<()> { Ok(()) },
            )
            .expect("host print import should link");
        let instance = linker
            .instantiate(&mut store, &module)
            .expect("instance should instantiate");
        let pre_main_bytes = if let Some(memory) = instance.get_memory(&mut store, "memory") {
            let mut bytes = vec![0; 32];
            memory
                .read(&mut store, WASM_STATIC_DATA_OFFSET, &mut bytes)
                .expect("pre-main bytes should read");
            Some(bytes)
        } else {
            None
        };
        let main = instance
            .get_typed_func::<(), i64>(&mut store, "main")
            .expect("main export should exist");
        let payload = main.call(&mut store, ()).expect("main should run");
        let WasmtimeVal::I32(result_tag) = instance
            .get_global(&mut store, WASM_RESULT_TAG_EXPORT)
            .expect("result_tag export should exist")
            .get(&mut store)
        else {
            panic!("result_tag export should be i32");
        };
        let WasmtimeVal::I32(completion_kind) = instance
            .get_global(&mut store, WASM_COMPLETION_KIND_EXPORT)
            .expect("completion_kind export should exist")
            .get(&mut store)
        else {
            panic!("completion_kind export should be i32");
        };
        let tag = WasmRuntimeValueTag::from_tag(result_tag).expect("result tag should decode");
        let post_main_prefix = if let Some(memory) = instance.get_memory(&mut store, "memory") {
            let mut bytes = vec![0; 32];
            memory
                .read(&mut store, WASM_STATIC_DATA_OFFSET, &mut bytes)
                .expect("post-main bytes should read");
            Some(bytes)
        } else {
            None
        };
        let bytes = if tag.value_kind() == ValueKind::String {
            let Some(memory) = instance.get_memory(&mut store, "memory") else {
                panic!("string result should export memory");
            };
            let offset = ((payload as u64) >> 32) as usize;
            let len = ((payload as u64) & 0xFFFF_FFFF) as usize;
            let mut bytes = vec![0; len];
            memory
                .read(&mut store, offset, &mut bytes)
                .expect("string bytes should read");
            Some(bytes)
        } else {
            None
        };
        (
            payload,
            tag,
            completion_kind,
            pre_main_bytes,
            post_main_prefix,
            bytes,
        )
    }

    fn wasm_import_export_names(bytes: &[u8]) -> (Vec<String>, Vec<String>) {
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        for payload in Parser::new(0).parse_all(bytes) {
            match payload.expect("wasm parse should succeed") {
                Payload::ImportSection(reader) => {
                    for import in reader {
                        match import.expect("import should decode") {
                            Imports::Single(_, import) => {
                                imports.push(format!("{}::{}", import.module, import.name));
                            }
                            Imports::Compact1 { module, items } => {
                                for item in items {
                                    let item = item.expect("compact import should decode");
                                    imports.push(format!("{module}::{}", item.name));
                                }
                            }
                            Imports::Compact2 { module, names, .. } => {
                                for name in names {
                                    imports.push(format!(
                                        "{module}::{}",
                                        name.expect("compact import name should decode")
                                    ));
                                }
                            }
                        }
                    }
                }
                Payload::ExportSection(reader) => {
                    for export in reader {
                        let export = export.expect("export should decode");
                        exports.push(export.name.to_string());
                    }
                }
                _ => {}
            }
        }
        imports.sort();
        exports.sort();
        (imports, exports)
    }

    #[test]
    fn wasm_backend_characterization_matrix_locks_public_surface_and_outcomes() {
        let cases = [
            (
                "arithmetic",
                "let x = 40; const y = 2; x + y;",
                "number(42",
                ValueKind::Number,
            ),
            (
                "string-concat",
                "let left = 'por'; let right = 'ffor'; left + right;",
                "string(porffor)",
                ValueKind::String,
            ),
            (
                "objects",
                "let o = { x: 1 }; o.y = 2; o.x + o.y;",
                "number(3",
                ValueKind::Number,
            ),
            (
                "caught-throw",
                "try { throw new TypeError('boom'); } catch (e) { e.name; }",
                "string(TypeError)",
                ValueKind::String,
            ),
        ];

        for (label, source, expected_note, expected_kind) in cases {
            let engine = engine();
            let unit = engine
                .compile_script(source, CompileOptions::default())
                .unwrap_or_else(|err| panic!("{label} should compile: {err:?}"));
            let artifact = engine
                .emit_wasm(&unit)
                .unwrap_or_else(|err| panic!("{label} should emit wasm: {err:?}"));
            let (imports, exports) = wasm_import_export_names(&artifact.bytes);
            // None of these characterization sources call `print`/`console`,
            // so the emitted module should carry no host imports at all —
            // the backend now elides unused host imports instead of always
            // pulling in `porf_host::print_line_utf8`, producing a leaner
            // public surface than the previously locked expectation.
            assert!(imports.is_empty(), "{label} imports: {imports:?}");
            for export in [
                "main",
                "memory",
                WASM_RESULT_TAG_EXPORT,
                WASM_COMPLETION_KIND_EXPORT,
                WASM_THROW_ERROR_NAME_EXPORT,
            ] {
                assert!(
                    exports.contains(&export.to_string()),
                    "{label} exports: {exports:?}"
                );
            }

            let outcome = engine
                .run_compiled_unit(
                    &unit,
                    source,
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| panic!("{label} should run: {err:?}"));
            assert!(
                outcome.note.contains(expected_note),
                "{label} note: {}",
                outcome.note
            );

            let (_, tag, completion_kind, _, _, _) = run_wasm_raw(source);
            assert_eq!(tag.value_kind(), expected_kind, "{label} result kind");
            assert_eq!(completion_kind, 0, "{label} completion kind");
        }

        let err = engine()
            .run_script(
                "throw new TypeError('boom');",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("uncaught throw should stay observable");
        assert!(
            err.message()
                .contains("uncaught throw: TypeError: wasm-aot completion: object(handle@"),
            "error: {err}"
        );
    }

    #[test]
    fn current_thread_wasm_aot_entry_runs_on_a_sized_worker() {
        let outcome = run_on_sized_stack(|| {
            engine().run_wasm_aot_script_on_current_thread(
                "40 + 2;",
                CompileOptions::default(),
                None,
            )
        })
        .expect("current-thread Wasm-AOT run should succeed");

        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(outcome.note.contains("number(42)"), "outcome: {outcome:?}");
    }

    #[test]
    fn current_thread_wasm_aot_entry_bypasses_memory_module_cache() {
        let source = "let currentThreadMemoryCacheBypass = 4815162342;";
        let unit = engine()
            .compile_script(source, CompileOptions::default())
            .expect("source should compile");
        let artifact = engine()
            .emit_wasm(&unit)
            .expect("compiled source should emit Wasm");
        assert!(
            !memory_wasm_module_is_cached(&artifact.bytes),
            "test artifact must not already be retained in the memory module cache"
        );

        run_on_sized_stack(|| {
            engine().run_wasm_aot_script_on_current_thread(source, CompileOptions::default(), None)
        })
        .expect("current-thread Wasm-AOT run should succeed");

        assert!(
            !memory_wasm_module_is_cached(&artifact.bytes),
            "current-thread Wasm-AOT runs must not retain their module in the memory cache"
        );
    }

    #[test]
    fn compile_script_marks_script_goal() {
        let unit = engine()
            .compile_script("let x = 1;", CompileOptions::default())
            .expect("script compile stub should succeed");
        assert_eq!(unit.source.goal, ParseGoal::Script);
        assert!(unit.ir.invariants.contains(&"direct-js-to-wasm-only"));
    }

    #[test]
    fn compile_module_marks_module_goal() {
        let unit = engine()
            .compile_module("export {};", CompileOptions::default())
            .expect("module compile stub should succeed");
        assert_eq!(unit.source.goal, ParseGoal::Module);
    }

    #[test]
    fn compile_script_preserves_structured_parse_diagnostic() {
        let err = engine()
            .compile_script("let x = ;", CompileOptions::default())
            .expect_err("malformed script should fail during parse");
        let diagnostic = err
            .parse_diagnostic()
            .expect("engine error should retain parse diagnostic");
        assert_eq!(
            diagnostic.kind,
            porffor_front::ParseDiagnosticKind::MalformedJavaScript
        );
        assert_eq!(diagnostic.phase, porffor_front::ParseDiagnosticPhase::Parse);
        assert_eq!(diagnostic.error_type, "SyntaxError");
        assert_eq!(diagnostic.code, "P_PARSE_MALFORMED");
        assert!(diagnostic.span.is_some());
    }

    #[test]
    fn engine_error_preserves_structured_ir_early_error_diagnostic() {
        let source_diagnostic =
            IrDiagnostic::early_error("E_TEST_EARLY", "SyntaxError", "early error: test", None);
        let err = EngineError::from_ir_diagnostic(source_diagnostic.clone());

        assert_eq!(err.message(), source_diagnostic.message);
        assert_eq!(err.ir_diagnostic(), Some(&source_diagnostic));
        assert!(err.parse_diagnostic().is_none());
    }

    #[test]
    fn compile_script_reports_front_end_early_error_diagnostic() {
        let err = engine()
            .compile_script(
                "({ __proto__: null, __proto__: {} });",
                CompileOptions::default(),
            )
            .expect_err("duplicate __proto__ prototype setters should be early error");
        let diagnostic = err
            .parse_diagnostic()
            .expect("engine error should retain front-end diagnostic");
        assert_eq!(diagnostic.code, "E_OBJECT_DUPLICATE_PROTO");
        assert_eq!(diagnostic.phase, porffor_front::ParseDiagnosticPhase::Early);
        assert_eq!(diagnostic.error_type, "SyntaxError");
        assert!(err.ir_diagnostic().is_none());
    }

    #[test]
    fn wasm_emit_succeeds_for_supported_script() {
        let unit = engine()
            .compile_script("1 + 1;", CompileOptions::default())
            .expect("script compile should succeed");
        let artifact = engine().emit_wasm(&unit).expect("wasm emit should succeed");
        assert_eq!(artifact.kind, ArtifactKind::Wasm);
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn wasm_backend_keeps_raw_string_payloads_stable() {
        let (payload, tag, completion, pre_main_bytes, post_main_prefix, bytes) =
            run_wasm_raw("\",\";");
        let mut expected_prefix = vec![b' '; 11];
        expected_prefix.extend_from_slice(b"\n: ,u");
        assert_eq!(tag.value_kind(), ValueKind::String);
        assert_eq!(completion, 0);
        assert_eq!(payload, (((4110u64) << 32) | 1) as i64);
        assert_eq!(
            pre_main_bytes.expect("pre-main bytes should exist")[..16].to_vec(),
            expected_prefix
        );
        assert_eq!(
            post_main_prefix.expect("post-main bytes should exist")[..16].to_vec(),
            expected_prefix
        );
        assert_eq!(bytes.expect("string bytes should exist"), b",".to_vec());
    }

    #[test]
    fn wasm_backend_renders_a_heap_bigint_normal_completion() {
        let outcome = engine()
            .run_script(
                "const buffer = new ArrayBuffer(8); const view = new DataView(buffer); \
                 for (let index = 0; index < 8; index++) view.setUint8(index, 255); \
                 view.getBigUint64(0);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("heap BigInt normal completion should run");

        assert_eq!(
            outcome.note,
            "wasm-aot completion: bigint(18446744073709551615n)"
        );
    }

    #[test]
    fn wasm_backend_renders_a_heap_bigint_throw_completion() {
        let error = engine()
            .run_script(
                "throw -340282366920938463481821351505477763073n;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("heap BigInt throw completion should remain observable");

        assert_eq!(
            error.message(),
            "uncaught throw: wasm-aot completion: \
             bigint(-340282366920938463481821351505477763073n)"
        );
    }

    #[test]
    fn wasm_backend_lowers_template_expression_legacy_octal_string_literal() {
        let outcome = engine()
            .run_script(
                "`${'\\07'}` === '\\u0007';",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("template expression legacy octal string literal should run");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_parses_decimal_exponents_with_combined_scale() {
        let outcome = engine()
            .run_script(
                "JSON.parse('1.1e-1') === 0.11 && parseFloat('1.1e-1') === 0.11 && parseFloat('1.23e1') === 12.3;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("decimal exponent parse should run");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_rounds_runtime_decimal_spans_exactly() {
        let outcome = engine()
            .run_script(
                r#"
var large = "23456789012E66";
var largeFraction = "1.234567890E+34";
var halfway = "1.00000000000000011102230246251565404236316680908203125";
var aboveHalfway = "1.00000000000000011102230246251565404236316680908203126";
var minimum = "4.9406564584124654e-324";
var maximum = "1.7976931348623157e308";
var overflow = "1.7976931348623159e308";
JSON.parse(large) === 2.3456789012e76 &&
JSON.parse(largeFraction) === 1.234567890e34 &&
JSON.parse("23456789012E66") === 2.3456789012e76 &&
JSON.parse("1.234567890E+34") === 1.234567890e34 &&
parseFloat(halfway + "suffix") === 1 &&
Number(aboveHalfway) === 1.0000000000000002 &&
JSON.parse(minimum, null) === 5e-324 &&
parseFloat(maximum) === 1.7976931348623157e308 &&
Number(overflow) === Infinity;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("runtime decimal conversion should run");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );

        for source in [
            "JSON.parse('-2e-324', null);",
            "parseFloat('-2e-324');",
            "Number('-2e-324');",
        ] {
            let (payload, tag, completion, _, _, _) = run_wasm_raw(source);
            assert_eq!(tag.value_kind(), ValueKind::Number, "source: {source}");
            assert_eq!(completion, 0, "source: {source}");
            assert_eq!(payload as u64, (-0.0f64).to_bits(), "source: {source}");
        }
    }

    #[test]
    fn wasm_backend_rounds_runtime_decimal_boundaries_exactly() {
        let cases = [
            ("JSON.parse('1e-4000');", 0.0f64.to_bits()),
            ("JSON.parse('2e-324');", 0.0f64.to_bits()),
            ("JSON.parse('3e-324');", 1),
            ("JSON.parse('2.225073858507201e-308');", (1_u64 << 52) - 1),
            (
                "JSON.parse('2.2250738585072012e-308');",
                f64::MIN_POSITIVE.to_bits(),
            ),
            ("JSON.parse('1.7976931348623157e308');", f64::MAX.to_bits()),
            ("JSON.parse('1.7976931348623158e308');", f64::MAX.to_bits()),
            (
                "JSON.parse('1.7976931348623159e308');",
                f64::INFINITY.to_bits(),
            ),
            ("JSON.parse('1e309');", f64::INFINITY.to_bits()),
        ];

        for (source, expected_bits) in cases {
            let (payload, tag, completion, _, _, _) = run_wasm_raw(source);
            assert_eq!(tag.value_kind(), ValueKind::Number, "source: {source}");
            assert_eq!(completion, 0, "source: {source}");
            assert_eq!(payload as u64, expected_bits, "source: {source}");
        }
    }

    #[test]
    fn wasm_backend_json_parse_duplicate_proto_literal_property() {
        let outcome = engine()
            .run_script(
                "var value = JSON.parse('{\"__proto__\": 1, \"__proto__\": 2}'); typeof value === 'object' && value.__proto__ === 2;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON.parse duplicate __proto__ should run");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_parse_structural_validator_accepts_nested_keywords() {
        let outcome = engine()
            .run_script(
                "function valid(text) { try { JSON.parse(text); return true; } catch (e) { return false; } } function invalid(text) { try { JSON.parse(text); return false; } catch (e) { return e instanceof SyntaxError; } } var zero = '{\"a\":0}'; var leading = '{\"a\":013}'; var trailing = '{\"a\":true} \"extra\"'; var doubleColon = '{\"a\"::true}'; valid('{\"a\":true}') && valid('{\"a\":false}') && valid('{\"a\":null}') && valid(zero) && invalid('{\"a\":tru}') && invalid('{\"a\":fals}') && invalid('{\"a\":nul}') && invalid('{\"a\":true,}') && invalid(leading) && invalid(trailing) && invalid(doubleColon);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON.parse nested keyword validation should run");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_parse_enforces_number_grammar_inside_structures() {
        let outcome = engine()
            .run_script(
                "function valid(text) { try { JSON.parse(text); return true; } catch (e) { return false; } } function invalid(text) { try { JSON.parse(text); return false; } catch (e) { return e instanceof SyntaxError; } } var accepted = ['0', '-0', '10', '0.5', '-12.75', '1e2', '1E+2', '1e-2', '[0,-0,10,0.5,1e2]', '{\"n\":-12.75e+2}', '{\"\\\\u0123\":5}', '[\".1\",\"00\",\"1e+\"]']; var rejected = ['-', '+', '00', '01', '1.', '1.e1', '.1', '1e', '1e+', '1e-', '[00]', '{\"n\":013}', '[1.]', '[1.e2]', '[.1]', '[1e]', '[1e+]', '[1e-]', '[0x14]']; var correct = true; for (var i = 0; i < accepted.length; i++) correct = correct && valid(accepted[i]); for (var i = 0; i < rejected.length; i++) correct = correct && invalid(rejected[i]); correct;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON.parse number grammar validation should run");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_parse_rejects_structural_trailing_commas() {
        let outcome = engine()
            .run_script(
                "function valid(text) { try { JSON.parse(text); return true; } catch (e) { return false; } } function invalid(text) { try { JSON.parse(text); return false; } catch (e) { return e instanceof SyntaxError; } } valid('[]') && valid('[1]') && valid('{}') && valid('{\"a\":1}') && invalid('[1,]') && invalid('[\"a\",]') && invalid('{,}') && invalid('{\"a\":1,}') && invalid('[{,}]') && invalid('[[1,]]') && invalid('[{\"a\":\"b\",}]');",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON.parse trailing-comma validation should run");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_parse_materializes_dynamic_composites() {
        let outcome = engine()
            .run_script(
                r#"
var text = '{"array":[null,true,false,-0,12.5e1,"line\\n\\u20ac","\\uD834\\uDF06","\\uD834"],"duplicate":1,' +
  '"__proto__":{"safe":true},"duplicate":2}';
var value = JSON.parse(text, null);
value.array.length === 8 &&
  value.array[0] === null && value.array[1] === true && value.array[2] === false &&
  1 / value.array[3] === -Infinity && value.array[4] === 125 &&
  value.array[5] === "line\n\u20ac" && value.duplicate === 2 &&
  value.array[6] === String.fromCharCode(0xD834, 0xDF06) &&
  value.array[7] === String.fromCharCode(0xD834) &&
  Object.keys(value.array).join(",") === "0,1,2,3,4,5,6,7" &&
  Object.keys(value).join(",") === "array,duplicate,__proto__" &&
  Object.getPrototypeOf(value) === Object.prototype &&
  Object.prototype.hasOwnProperty.call(value, "__proto__") &&
  value.__proto__.safe === true;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("dynamic JSON arrays and objects should materialize");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_parse_uses_an_unbounded_dynamic_nesting_stack() {
        let outcome = engine()
            .run_script(
                r#"
var text = "0";
for (var i = 0; i < 160; i++) text = "[" + text + "]";
var value = JSON.parse(text);
for (var j = 0; j < 160; j++) value = value[0];
value === 0;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("dynamic JSON nesting should not be limited by validator bit masks");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_parse_grows_dynamic_arrays_amortized() {
        let outcome = engine()
            .run_script(
                r#"
var text = "[";
for (var i = 0; i < 257; i++) text += (i === 0 ? "" : ",") + i;
text += "]";
var value = JSON.parse(text, false);
value.length === 257 && value[0] === 0 && value[127] === 127 && value[256] === 256;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("dynamic JSON arrays should grow without reparsing or fixed limits");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_wasm_aot_epoch_interruption_bounds_infinite_loop() {
        // CRITICAL correctness check for the in-process default execution
        // path (see `run_case_entry` in porffor-test262 and
        // `run_with_wasm_aot_inner` above): a genuinely hanging Wasm-AOT
        // module must trap out on its own on a bounded schedule instead of
        // hanging this thread forever, since nothing else (no child process
        // to kill) protects against it anymore.
        let start = std::time::Instant::now();
        let err = engine()
            .run_script(
                "while (true) {}",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(300),
                    ..RunOptions::default()
                },
            )
            .expect_err("an infinite loop must trap via epoch interruption, not hang");
        let elapsed = start.elapsed();
        assert!(
            err.message().contains("timeout exceeded"),
            "expected a timeout-classified error, got: {err}"
        );
        // Epoch ticks are WASM_EPOCH_TICK_MS (100ms) apart, so a 300ms bound
        // should trip within a few ticks of that -- generous upper bound
        // here to stay robust under a loaded CI/sandbox scheduler while
        // still proving this is bounded, not "never returns".
        assert!(
            elapsed < std::time::Duration::from_secs(10),
            "epoch interruption should trap promptly, took {elapsed:?}"
        );
    }

    #[test]
    fn wasm_backend_wasm_aot_finishes_near_timeout_deadline_is_not_falsely_killed() {
        // A case that legitimately finishes just under its deadline must
        // still be reported as a success, not spuriously classified as a
        // timeout because it happened to run close to the bound. Loop a
        // bounded, deterministic amount of work rather than sleeping (Wasm
        // has no sleep primitive here); this keeps the check fast while
        // still exercising many epoch-check back-edges before returning
        // normally, well inside a generous timeout bound.
        let outcome = engine()
            .run_script(
                "var sum = 0; for (var i = 0; i < 200000; i++) { sum += i; } sum;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("a legitimately-finishing case must not be falsely killed by the timeout");
        assert!(outcome.note.contains("number("), "note: {}", outcome.note);
    }

    #[test]
    fn wasm_backend_json_parse_reviver_uses_root_wrapper_this() {
        let outcome = engine()
            .run_script(
                "var wrapper; JSON.parse('2', function() { wrapper = this; }); var isEnumerable = Function.prototype.call.bind(Object.prototype.propertyIsEnumerable); function enumCheck(obj, name) { var seen = false; for (var key in obj) { if (key === name) { seen = true; break; } } return seen && Object.prototype.hasOwnProperty.call(obj, name) && isEnumerable(obj, name); } function configurableCheck(obj, name) { delete obj[name]; return !Object.prototype.hasOwnProperty.call(obj, name); } typeof wrapper === 'object' && Object.getPrototypeOf(wrapper) === Object.prototype && Object.getOwnPropertyNames(wrapper).length === 1 && wrapper[''] === 2 && enumCheck(wrapper, '') && configurableCheck(wrapper, '');",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON.parse reviver wrapper should run");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_parse_reviver_context_sources_for_static_json() {
        let outcome = engine()
            .run_script(
                "var ok = true; var result = JSON.parse('[1.0,\"2\",true,null,{\"x\":1}]', function(k, v, c) { if (k === '0') ok = ok && c.source === '1.0'; if (k === '1') ok = ok && c.source === '\"2\"'; if (k === '2') ok = ok && c.source === 'true'; if (k === '3') ok = ok && c.source === 'null'; if (k === 'x') ok = ok && c.source === '1'; if (k === '4') ok = ok && typeof v === 'object' && v.x === 1 && Object.getOwnPropertyNames(c).length === 0; if (k === '') ok = ok && Object.getOwnPropertyNames(c).length === 0; return v; }); ok && result.length === 5 && result[0] === 1 && result[1] === '2' && result[2] === true && result[3] === null && result[4].x === 1;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON.parse reviver context sources should run");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_parse_dynamic_reviver_walks_post_order_with_exact_sources() {
        let outcome = engine()
            .run_script(
                r#"
var calls = "";
var sources = "";
function parse(text) {
  return JSON.parse(text, function(key, value, context) {
    calls += (calls === "" ? "" : ",") + key;
    sources += (sources === "" ? "" : ",") +
      (Object.prototype.hasOwnProperty.call(context, "source") ? context.source : "-");
    return value;
  });
}
var result = parse(' [1.0,{"x":"two"}] ');
calls === "0,x,1," && sources === '1.0,"two",-,-' &&
  result[0] === 1 && result[1].x === "two";
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("dynamic JSON reviver traversal should run");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_parse_dynamic_reviver_accepts_callable_proxy() {
        let outcome = engine()
            .run_script(
                r#"
var calls = 0;
var sourceIsExact = false;
var reviver = new Proxy(function(key, value, context) {
  calls += 1;
  sourceIsExact = key === "" && context.source === "1.0";
  return value;
}, {});
function parse(text, callback) { return JSON.parse(text, callback); }
var result = parse("  1.0  ", reviver);
result === 1 && calls === 1 && sourceIsExact;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("callable Proxy JSON reviver should run");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_parse_dynamic_reviver_deletes_and_replaces_properties() {
        let outcome = engine()
            .run_script(
                r#"
function parse(text) {
  return JSON.parse(text, function(key, value) {
    if (key === "drop") return undefined;
    if (key === "0") return value + 10;
    if (key === "") return { wrapped: value };
    return value;
  });
}
var result = parse('{"keep":[1,2],"drop":3}');
result.wrapped.keep[0] === 11 && result.wrapped.keep[1] === 2 &&
  !Object.prototype.hasOwnProperty.call(result.wrapped, "drop");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("dynamic JSON reviver deletion and replacement should run");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_parse_dynamic_reviver_observes_forward_holder_mutation() {
        let outcome = engine()
            .run_script(
                r#"
var replacement = { deep: { marker: 2 } };
var calls = "";
function parse(text) {
  return JSON.parse(text, function(key, value) {
    calls += (calls === "" ? "" : ",") + key;
    if (key === "0") this[1] = replacement;
    return value;
  });
}
var result = parse('[0,{"old":1}]');
calls === "0,marker,deep,1," && result[1] === replacement &&
  result[1].deep === replacement.deep && result[1].deep.marker === 2;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("dynamic JSON reviver forward holder mutation should run");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_parse_dynamic_reviver_propagates_abrupt_completion() {
        let outcome = engine()
            .run_script(
                r#"
var calls = 0;
var objectSentinel = {};
var primitiveSentinel = "stop";
function parseObjectThrow(text) {
  return JSON.parse(text, function(key, value) {
    calls += 1;
    if (key === "0") throw objectSentinel;
    return value;
  });
}
function parsePrimitiveThrow(text) {
  return JSON.parse(text, function(key, value) {
    calls += 1;
    if (key === "0") throw primitiveSentinel;
    return value;
  });
}
var objectIdentityPreserved = false;
try { parseObjectThrow("[1,2]"); } catch (error) {
  objectIdentityPreserved = error === objectSentinel;
}
var primitiveIdentityPreserved = false;
try { parsePrimitiveThrow("[1,2]"); } catch (error) {
  primitiveIdentityPreserved = error === primitiveSentinel;
}
objectIdentityPreserved && primitiveIdentityPreserved && calls === 2;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("dynamic JSON reviver throw should propagate");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_parse_dynamic_reviver_preserves_array_accessor_throw_identity() {
        let outcome = engine()
            .run_script(
                r#"
var accessorSentinel = {};
function parseWithAccessorThrow() {
  return JSON.parse("[0,0]", function() {
    Object.defineProperty(this, "1", {
      get: function() { throw accessorSentinel; }
    });
  });
}
var identityPreserved = false;
try { parseWithAccessorThrow(); } catch (error) {
  identityPreserved = error === accessorSentinel;
}
identityPreserved;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("dynamic JSON reviver array accessor throw should propagate");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_stringify_snapshots_replacer_array_before_serializing_values() {
        let outcome = engine()
            .run_script(
                r#"
var conversions = 0;
var accesses = [];
var boxedKey = new String("a");
boxedKey.toString = function() {
  conversions += 1;
  return "a";
};
var target = [boxedKey];
var replacer = new Proxy(target, {
  get: function(target, key) {
    accesses.push(key);
    return target[key];
  }
});
var value = {};
Object.defineProperty(value, "a", {
  enumerable: true,
  get: function() {
    target[0] = "b";
    return { a: 1, b: 2 };
  }
});
[
  JSON.stringify(value, replacer),
  accesses.join(","),
  conversions
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON stringify should snapshot an array replacer once");
        assert!(
            outcome
                .note
                .contains("string({\"a\":{\"a\":1}}|length,0|1)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_stringify_normalizes_property_list_entries_in_first_seen_order() {
        let outcome = engine()
            .run_script(
                r#"
var conversions = [];
var boxedString = new String("ignored");
boxedString.toString = function() {
  conversions.push("string");
  return "b";
};
var boxedNumber = new Number(7);
Object.setPrototypeOf(boxedNumber, {
  toString: function() {
    conversions.push("number");
    return "a";
  }
});
var replacer = [
  "c", boxedString, boxedNumber, "c", 1, -0, NaN, Infinity,
  undefined, null, true, {}, Symbol("ignored")
];
var value = { a: 1, b: 2, c: 3, 1: 4, 0: 5, NaN: 6, Infinity: 7 };
[
  JSON.stringify(value, replacer),
  conversions.join(","),
  JSON.stringify({ a: 1 }, function(key, current) {
    return key === "a" ? current + 1 : current;
  })
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON stringify should normalize property list entries once");
        assert!(
            outcome.note.contains(
                "string({\"c\":3,\"b\":2,\"a\":1,\"1\":4,\"0\":5,\"NaN\":6,\"Infinity\":7}|string,number|{\"a\":2})"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_stringify_calls_proxy_replacer_with_exact_holders_and_arguments() {
        let outcome = engine()
            .run_script(
                r#"
var child = { value: 1 };
var input = { child: child };
var rootWrapper;
var calls = [];
var holders = [];
var applyHolders = [];
var target = function(key, value) {
  calls.push(key);
  if (key === "") {
    rootWrapper = this;
    holders.push(this !== input && this[""] === input);
  } else if (key === "child") {
    holders.push(this === input);
  } else if (key === "value") {
    holders.push(this === child);
    return value + 1;
  }
  return value;
};
var replacer = new Proxy(target, {
  apply: function(target, thisArg, args) {
    if (args[0] === "") {
      applyHolders.push(thisArg !== input && thisArg[""] === input);
    } else if (args[0] === "child") {
      applyHolders.push(thisArg === input);
    } else if (args[0] === "value") {
      applyHolders.push(thisArg === child);
    }
    return target.apply(thisArg, args);
  }
});
var result = JSON.stringify(input, replacer);

var sentinel = {};
var abruptIdentity = false;
try {
  JSON.stringify({}, new Proxy(function() {}, {
    apply: function() { throw sentinel; }
  }));
} catch (error) {
  abruptIdentity = error === sentinel;
}

var revoked = Proxy.revocable(function() {}, {});
revoked.revoke();
var revokedTypeError = false;
try {
  JSON.stringify({}, revoked.proxy);
} catch (error) {
  revokedTypeError = error instanceof TypeError;
}

[
  result,
  calls.join(","),
  holders.join(","),
  applyHolders.join(","),
  Object.getPrototypeOf(rootWrapper) === Object.prototype,
  abruptIdentity,
  revokedTypeError
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON stringify should call Proxy replacers with exact holders");
        assert!(
            outcome.note.contains(
                "string({\"child\":{\"value\":2}}|,child,value|true,true,true|true,true,true|true|true|true)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_stringify_calls_proxy_to_json_and_omits_callable_proxies() {
        let outcome = engine()
            .run_script(
                r#"
var callable = new Proxy(function() {}, {});
var order = [];
var value = {};
value.toJSON = new Proxy(function(key) {
  order.push("toJSON:" + key);
  return { kept: 1, dropped: callable };
}, {
  apply: function(target, thisArg, args) {
    order.push("toJSONApply:" + args[0] + ":" + (thisArg === value));
    return target.apply(thisArg, args);
  }
});
var result = JSON.stringify(value, function(key, current) {
  order.push("replacer:" + key);
  return key === "kept" ? current + 1 : current;
});

var getterSentinel = {};
var getterIdentity = false;
var replacerCalled = false;
var abruptToJSON = {};
Object.defineProperty(abruptToJSON, "toJSON", {
  get: function() { throw getterSentinel; }
});
try {
  JSON.stringify(abruptToJSON, function(key, current) {
    replacerCalled = true;
    return current;
  });
} catch (error) {
  getterIdentity = error === getterSentinel;
}

var hole = [];
hole.length = 1;
[
  result,
  order.join(","),
  JSON.stringify(callable) === undefined,
  JSON.stringify([callable]),
  JSON.stringify({ callable: callable }),
  JSON.stringify(0, function() { return callable; }) === undefined,
  JSON.stringify([0], function(key, current) {
    return key === "0" ? callable : current;
  }),
  JSON.stringify({ value: 0 }, function(key, current) {
    return key === "value" ? callable : current;
  }),
  JSON.stringify(hole, function(key, current) {
    return key === "0" ? "filled" : current;
  }),
  getterIdentity,
  replacerCalled
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON stringify should call Proxy toJSON and omit callable Proxies");
        assert!(
            outcome.note.contains(
                "string({\"kept\":2}|toJSONApply::true,toJSON:,replacer:,replacer:kept,replacer:dropped|true|[null]|{}|true|[null]|{}|[\"filled\"]|true|false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_stringify_space_coerces_boxed_values_once_after_replacer_setup() {
        let outcome = engine()
            .run_script(
                r#"
var order = [];
var replacer = new Proxy(["outer"], {
  get: function(target, key) {
    if (key === "length" || key === "0") order.push("replacer:" + key);
    return target[key];
  }
});
var numberCalls = 0;
var numberReceiver = false;
var numberArgs = "";
var numberSpace = new Number(1);
var numberHook = function(hint) {
  numberCalls++;
  order.push("space:" + hint);
  return "2.9";
};
numberSpace[Symbol.toPrimitive] = new Proxy(numberHook, {
  apply: function(target, receiver, args) {
    numberReceiver = receiver === numberSpace;
    numberArgs = args.length + ":" + args[0];
    return target.apply(receiver, args);
  }
});
numberSpace.valueOf = function() { throw new Error("valueOf should not run"); };
var numberResult = JSON.stringify(
  { outer: { inner: 1 }, dropped: 2 },
  replacer,
  numberSpace
);

var stringCalls = 0;
var stringSpace = new String("ignored");
stringSpace.toString = function() {
  stringCalls++;
  return 3;
};
stringSpace.valueOf = function() { throw new Error("valueOf should not run"); };
var stringResult = JSON.stringify({ a: 1 }, null, stringSpace);

[
  numberResult === '{\n  "outer": {}\n}',
  order.join(","),
  numberCalls,
  numberReceiver,
  numberArgs,
  stringResult === '{\n3"a": 1\n}',
  stringCalls
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON stringify should coerce boxed space once after replacer setup");
        assert!(
            outcome.note.contains(
                "string(true|replacer:length,replacer:0,space:number|1|true|1:number|true|1)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_stringify_space_truncates_utf16_code_units() {
        let outcome = engine()
            .run_script(
                r#"
var bmpGap = "éééééé";
var bmpExpected = "{\n" + bmpGap + '"a": {\n' + bmpGap + bmpGap +
  '"b": 1\n' + bmpGap + "}\n}";
var astralGap = "12345678😀";
var astralExpected = "{\n" + astralGap + '"a": 1\n}';
var highSurrogateGap = "123456789\uD83D";
var highSurrogateExpected = "{\n" + highSurrogateGap + '"a": 1\n}';
[
  JSON.stringify({ a: { b: 1 } }, null, bmpGap) === bmpExpected,
  JSON.stringify({ a: 1 }, null, "12345678😀X") === astralExpected,
  JSON.stringify({ a: 1 }, null, "123456789😀") === highSurrogateExpected
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON stringify should truncate space to ten UTF-16 code units");
        assert!(
            outcome.note.contains("string(true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_stringify_space_preserves_abrupt_identity_and_error_realm() {
        let outcome = engine()
            .run_script(
                r#"
var getterSentinel = {};
var getterIdentity = false;
var getterSpace = new String("gap");
Object.defineProperty(getterSpace, "toString", {
  get: function() { throw getterSentinel; }
});
try {
  JSON.stringify({}, null, getterSpace);
} catch (error) {
  getterIdentity = error === getterSentinel;
}

var applySentinel = {};
var applyIdentity = false;
var applySpace = new Number(1);
applySpace.valueOf = new Proxy(function() { return 1; }, {
  apply: function() { throw applySentinel; }
});
try {
  JSON.stringify({}, null, applySpace);
} catch (error) {
  applyIdentity = error === applySentinel;
}

var revoked = Proxy.revocable(function() { return 1; }, {});
var revokedSpace = new Number(1);
revokedSpace.valueOf = revoked.proxy;
revoked.revoke();
var revokedTypeError = false;
try {
  JSON.stringify({}, null, revokedSpace);
} catch (error) {
  revokedTypeError = error instanceof TypeError;
}

var other = __porfCreateRealm().global;
var exhaustedSpace = new Number(1);
exhaustedSpace.valueOf = function() { return {}; };
exhaustedSpace.toString = function() { return {}; };
var exhaustedRealm = false;
try {
  other.JSON.stringify({}, null, exhaustedSpace);
} catch (error) {
  exhaustedRealm = Object.getPrototypeOf(error) === other.TypeError.prototype;
}

var symbolSpace = new String("ignored");
symbolSpace.toString = function() { return Symbol("gap"); };
var symbolRealm = false;
try {
  other.JSON.stringify({}, null, symbolSpace);
} catch (error) {
  symbolRealm = Object.getPrototypeOf(error) === other.TypeError.prototype;
}

var nonCallableSpace = new Number(1);
nonCallableSpace[Symbol.toPrimitive] = 0;
var nonCallableRealm = false;
try {
  other.JSON.stringify({}, null, nonCallableSpace);
} catch (error) {
  nonCallableRealm = Object.getPrototypeOf(error) === other.TypeError.prototype;
}

var proxyGets = 0;
var ignoredProxy = new Proxy(new Number(2), {
  get: function() {
    proxyGets++;
    throw new Error("Proxy space should not be coerced");
  }
});
var ignoredResult = JSON.stringify({ a: 1 }, null, ignoredProxy);

[
  getterIdentity,
  applyIdentity,
  revokedTypeError,
  exhaustedRealm,
  symbolRealm,
  nonCallableRealm,
  ignoredResult === '{"a":1}',
  proxyGets
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON stringify space coercion should preserve abrupt identity and realm");
        assert!(
            outcome
                .note
                .contains("string(true|true|true|true|true|true|true|0)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_stringify_escapes_ascii_property_names_and_values() {
        let outcome = engine()
            .run_script(
                r#"
var input = "Az\"\\\x00\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x1f";
var expected = "\"Az\\\"\\\\\\u0000\\u0007\\b\\t\\n\\u000b\\f\\r\\u000e\\u001f\"";
var object = {};
object[input] = input;
JSON.stringify(input) === expected &&
  JSON.stringify(object) === "{" + expected + ":" + expected + "}";
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON stringify should escape ASCII property names and values");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_stringify_escapes_only_unpaired_utf16_surrogates() {
        let outcome = engine()
            .run_script(
                r#"
var high = "\uD800";
var low = "\uDC00";
var object = {};
object[high] = low;
var checks = [
  JSON.stringify(high) === '"\\ud800"',
  JSON.stringify("\uDBFF") === '"\\udbff"',
  JSON.stringify(low) === '"\\udc00"',
  JSON.stringify("\uDFFF") === '"\\udfff"',
  JSON.stringify("\uD834\uDF06") === '"𝌆"',
  JSON.stringify(high + low) === '"𐀀"',
  JSON.stringify(low + high) === '"\\udc00\\ud800"',
  JSON.stringify(high + "x") === '"\\ud800x"',
  JSON.stringify(object) === '{"\\ud800":"\\udc00"}',
  JSON.stringify({ toJSON: function() { return "\uD801"; } }) === '"\\ud801"',
  JSON.stringify(0, function(key, value) {
    return key === "" ? "\uDFFE" : value;
  }) === '"\\udffe"',
  JSON.stringify({
    toJSON: function() { return new String(high + low); }
  }) === '"𐀀"',
  JSON.stringify(JSON.rawJSON('"\\ud800"')) === '"\\ud800"'
];
var allPassed = true;
for (var i = 0; i < checks.length; i++) {
  if (!checks[i]) allPassed = false;
}
allPassed;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON stringify should escape only unpaired UTF-16 surrogates");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_stringify_handles_heap_bigints_and_uses_its_defining_realm() {
        let outcome = engine()
            .run_script(
                r#"
var huge = 340282366920938463463374607431768211456n;
var negativeHuge = -340282366920938463463374607431768211456n;
var decimal = "340282366920938463463374607431768211456";
var other = __porfCreateRealm().global;
var crossRealmWrapped = other.Object(other.BigInt(100));
var crossRealmWrappedPrototype = Object.getPrototypeOf(crossRealmWrapped) === other.BigInt.prototype;
var mainRealmThrow = false;
var otherRealmThrow = false;
var crossRealmWrappedThrow = false;
var crossRealmWrappedConstructor = false;
try { JSON.stringify(huge); } catch (error) {
  mainRealmThrow = error instanceof TypeError;
}
try { other.JSON.stringify(huge); } catch (error) {
  otherRealmThrow = error instanceof other.TypeError && !(error instanceof TypeError);
}
try { JSON.stringify(crossRealmWrapped); } catch (error) {
  crossRealmWrappedThrow = error instanceof TypeError;
  crossRealmWrappedConstructor = error.constructor === TypeError;
}
BigInt.prototype.toJSON = function() { return this.toString(); };
var mainPrimitive = JSON.stringify(huge);
var mainBoxed = JSON.stringify(Object(huge));
var otherStillThrows = false;
try { other.JSON.stringify(huge); } catch (error) {
  otherStillThrows = error instanceof other.TypeError;
}
other.BigInt.prototype.toJSON = function() { return this.toString(); };
[
  mainRealmThrow,
  otherRealmThrow,
  mainPrimitive === '"' + decimal + '"',
  mainBoxed === '"' + decimal + '"',
  otherStillThrows,
  other.JSON.stringify(huge) === '"' + decimal + '"',
  crossRealmWrappedPrototype,
  crossRealmWrappedThrow,
  crossRealmWrappedConstructor,
  JSON.stringify(crossRealmWrapped) === '"100"',
  JSON.stringify(huge, function(key, value) {
    return typeof value === "bigint" ? value.toString() : value;
  }) === '"' + decimal + '"',
  Object(huge).valueOf() === huge,
  huge.toString(16) === "100000000000000000000000000000000",
  JSON.stringify(negativeHuge) === '"-' + decimal + '"',
  negativeHuge.toString(16) === "-100000000000000000000000000000000"
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON stringify should handle heap BigInts in its defining realm");
        assert!(
            outcome.note.contains(
                "string(true|true|true|true|true|true|true|true|true|true|true|true|true|true|true)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_raw_json_preserves_source_and_uses_an_unforgeable_frozen_brand() {
        let outcome = engine()
            .run_script(
                r#"
var raw = JSON.rawJSON("1e400");
var escaped = JSON.rawJSON('"\\u0061"');
var descriptor = Object.getOwnPropertyDescriptor(raw, "rawJSON");
var proxy = new Proxy(raw, {});
var inherited = Object.create(raw);
var fake = { rawJSON: "0" };
var mutationThrows = false;
try { Object.defineProperty(raw, "rawJSON", { value: "0" }); } catch (error) {
  mutationThrows = error instanceof TypeError;
}
[
  Object.getPrototypeOf(raw) === null,
  Object.isFrozen(raw),
  Object.isExtensible(raw),
  descriptor.value,
  descriptor.writable,
  descriptor.enumerable,
  descriptor.configurable,
  JSON.isRawJSON(raw),
  JSON.isRawJSON(proxy),
  JSON.isRawJSON(inherited),
  JSON.isRawJSON(fake),
  JSON.stringify(raw),
  JSON.stringify(escaped),
  JSON.stringify({ raw: raw }),
  JSON.stringify({ 42: JSON.rawJSON("37") }),
  JSON.stringify([escaped, raw]),
  JSON.stringify(proxy),
  mutationThrows,
  raw.rawJSON
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON raw wrappers should preserve source and remain unforgeable");
        assert!(
            outcome.note.contains(
                "string(true|true|false|1e400|false|true|false|true|false|false|false|1e400|\"\\u0061\"|{\"raw\":1e400}|{\"42\":37}|[\"\\u0061\",1e400]|{\"rawJSON\":\"1e400\"}|true|1e400)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_json_raw_json_validates_primitive_text_after_coercion_in_its_realm() {
        let outcome = engine()
            .run_script(
                r#"
var invalidTexts = [
  "", " true", "true ", "\t0", "0\n", "{}", "[]", "undefined", "NaN",
  "Infinity", "-Infinity", "01", "1.", "1e", "1e+", "--1", "+1", ".1",
  "truefalse", "nul", "\"unterminated", "\"bad\\x\"", "\"ok\"x"
];
var syntaxErrors = 0;
for (var i = 0; i < invalidTexts.length; i++) {
  try { JSON.rawJSON(invalidTexts[i]); } catch (error) {
    if (error instanceof SyntaxError) syntaxErrors += 1;
  }
}
var coercions = 0;
var coerced = JSON.rawJSON({
  toString: function() { coercions += 1; return "42"; }
});
var sentinel = {};
var abruptIdentity = false;
try {
  JSON.rawJSON({ toString: function() { throw sentinel; } });
} catch (error) {
  abruptIdentity = error === sentinel;
}
var symbolTypeError = false;
try { JSON.rawJSON(Symbol("1")); } catch (error) {
  symbolTypeError = error instanceof TypeError;
}
var other = __porfCreateRealm().global;
var otherSyntaxRealm = false;
try { other.JSON.rawJSON("[]"); } catch (error) {
  otherSyntaxRealm = Object.getPrototypeOf(error) === other.SyntaxError.prototype;
}
var otherTypeRealm = false;
var otherRawJSON = other.JSON.rawJSON;
try { otherRawJSON(Symbol("1")); } catch (error) {
  otherTypeRealm = Object.getPrototypeOf(error) === other.TypeError.prototype;
}
[
  syntaxErrors,
  invalidTexts.length,
  coercions,
  coerced.rawJSON,
  abruptIdentity,
  symbolTypeError,
  otherSyntaxRealm,
  otherTypeRealm,
  JSON.isRawJSON(other.JSON.rawJSON("1")),
  other.JSON.isRawJSON(JSON.rawJSON("2"))
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON.rawJSON should validate primitive text after observable coercion");
        assert!(
            outcome
                .note
                .contains("string(23|23|1|42|true|true|true|true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_object_has_own_checks_to_object_before_key() {
        let outcome = engine()
            .run_script(
                "var calls = 0; var key = { toString: function() { calls = calls + 1; throw 'key'; } }; var ok = false; try { Object.hasOwn(undefined, key); } catch (e) { ok = e.name === 'TypeError'; } ok && calls === 0 && Object.hasOwn({x: 1}, 'x');",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Object.hasOwn ToObject ordering case should run");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_get_own_property_names_to_object_string() {
        let outcome = engine()
            .run_script(
                "var empty = Object.getOwnPropertyNames(''); var text = Object.getOwnPropertyNames('ab'); var boxed = new String('abc'); boxed[5] = 'de'; var boxedNames = Object.getOwnPropertyNames(boxed); var arrNames = Object.getOwnPropertyNames([0, 1, 2]); var arrWithAccessor = [0, 1, 2]; Object.defineProperty(arrWithAccessor, 'ownProperty', { get: function() { return 'ownArray'; }, configurable: true }); var arrAccessorNames = Object.getOwnPropertyNames(arrWithAccessor); var accessorFound = false; for (var p in arrAccessorNames) { if (arrAccessorNames[p] === 'ownProperty') { accessorFound = true; } } var arrWithDot = [0, 1, 2]; arrWithDot.ownProperty = 'ownArray'; var arrDotNames = Object.getOwnPropertyNames(arrWithDot); var arrOrder = []; arrOrder.a = 1; Object.defineProperty(arrOrder, 'length', { value: 2 }); var arrOrderNames = Object.getOwnPropertyNames(arrOrder); var threw = false; try { Object.getOwnPropertyNames(undefined); } catch (e) { threw = e.name === 'TypeError'; } empty.length === 1 && empty[0] === 'length' && text.length === 3 && text[0] === '0' && text[1] === '1' && text[2] === 'length' && boxedNames.length === 5 && boxedNames[0] === '0' && boxedNames[1] === '1' && boxedNames[2] === '2' && boxedNames[3] === '5' && boxedNames[4] === 'length' && arrNames.length === 4 && arrNames[0] === '0' && arrNames[1] === '1' && arrNames[2] === '2' && arrNames[3] === 'length' && arrAccessorNames.length === 5 && arrAccessorNames[0] === '0' && arrAccessorNames[1] === '1' && arrAccessorNames[2] === '2' && arrAccessorNames[3] === 'length' && arrAccessorNames[4] === 'ownProperty' && accessorFound && arrDotNames.length === 5 && arrDotNames[0] === '0' && arrDotNames[1] === '1' && arrDotNames[2] === '2' && arrDotNames[3] === 'length' && arrDotNames[4] === 'ownProperty' && arrWithDot.ownProperty === 'ownArray' && arrOrderNames.length === 2 && arrOrderNames[0] === 'length' && arrOrderNames[1] === 'a' && threw;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Object.getOwnPropertyNames ToObject string case should run");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_catches_arraybuffer_dataview_constructor_toindex_abrupts() {
        for source in [
            "try { new ArrayBuffer({ valueOf: function() { throw \"x\"; } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
            "try { new ArrayBuffer(0, { maxByteLength: { valueOf: function() { throw \"x\"; } } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
            "try { new DataView(new ArrayBuffer(8), -1); 0; } catch (e) { e.name === \"RangeError\" ? 123 : 1; }",
            "try { new DataView(new ArrayBuffer(8), 0, -1); 0; } catch (e) { e.name === \"RangeError\" ? 123 : 1; }",
            "try { new Uint8Array(new ArrayBuffer(8), { valueOf: function() { throw \"x\"; } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
            "try { new Uint8Array(new ArrayBuffer(8), 0, { valueOf: function() { throw \"x\"; } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
            "var view = new DataView(new ArrayBuffer(8)); try { view.getUint8({ valueOf: function() { throw \"x\"; } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
            "var view = new DataView(new ArrayBuffer(8)); try { view.setUint16({ valueOf: function() { throw \"x\"; } }, 1); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
            "var view = new DataView(new ArrayBuffer(8)); try { view.getBigInt64({ valueOf: function() { throw \"x\"; } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
            "try { new ArrayBuffer(1, { maxByteLength: 4 }).resize({ valueOf: function() { throw \"x\"; } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
            "try { new ArrayBuffer(1).transfer({ valueOf: function() { throw \"x\"; } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| {
                    panic!("constructor ToIndex abrupt case should run for `{source}`: {err:?}")
                });
            assert!(
                outcome.note.contains("number(123"),
                "source: {source}, note: {}",
                outcome.note
            );
        }
    }

    #[test]
    fn wasm_backend_typed_array_buffer_arguments_use_private_current_state() {
        let source = r#"
const buffer = new ArrayBuffer(8);
buffer.$ArrayBufferDataPtr = 0;
buffer.$ArrayBufferByteLength = 0;
Object.defineProperty(buffer, "length", {
  get() { throw "buffer length property must not be read"; }
});
Object.defineProperty(buffer, Symbol.iterator, {
  get() { throw "buffer iterator property must not be read"; }
});
const privateView = new Uint16Array(buffer);

const forged = {
  $ArrayBufferDataPtr: 1,
  $ArrayBufferByteLength: 8,
  length: 2,
  0: 7,
  1: 8
};
const forgedView = new Uint8Array(forged);

const detached = new ArrayBuffer(4);
const conversionOrder = [];
let detachedTypeError = false;
try {
  new Uint8Array(
    detached,
    { valueOf() { conversionOrder.push("offset"); __porfDetachArrayBuffer(detached); return 0; } },
    { valueOf() { conversionOrder.push("length"); return 1; } }
  );
} catch (error) {
  detachedTypeError = error instanceof TypeError;
}

const shrinking = new ArrayBuffer(8, { maxByteLength: 8 });
let shrinkRangeError = false;
try {
  new Uint8Array(shrinking, 0, {
    valueOf() { shrinking.resize(1); return 2; }
  });
} catch (error) {
  shrinkRangeError = error instanceof RangeError;
}

const growing = new ArrayBuffer(1, { maxByteLength: 4 });
const grownView = new Uint8Array(growing, 0, {
  valueOf() { growing.resize(4); return 4; }
});
const tracking = new Uint8Array(growing, 0, undefined);
growing.resize(2);
const fixed = new Uint8Array(growing, 0, 1);
growing.resize(4);

const shared = new BigInt64Array(new SharedArrayBuffer(16), 8);
[
  privateView.length,
  forgedView.join(","),
  conversionOrder.join(","),
  detachedTypeError,
  shrinkRangeError,
  grownView.length,
  tracking.length,
  fixed.length,
  shared.length
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray buffer arguments should use branded private current state");
        assert!(
            outcome
                .note
                .contains("string(4|7,8|offset,length|true|true|4|4|1|1)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_length_reports_backing_store_allocation_failure() {
        let source = r#"
let threwRangeError = false;
try {
  new Uint8Array(1024 * 1024 * 1024);
} catch (error) {
  threwRangeError = error instanceof RangeError;
}
threwRangeError;
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray backing-store allocation failure should be catchable");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_length_zeroes_reused_heap_storage() {
        let source = r#"
let text = "12345678901234567890";
text += "1234567890";
const parsed = Number(text);
const values = new Uint8Array(8);
parsed > 0 &&
  values[0] === 0 && values[1] === 0 && values[2] === 0 && values[3] === 0 &&
  values[4] === 0 && values[5] === 0 && values[6] === 0 && values[7] === 0;
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray length construction should zero reused heap storage");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_bigint_typed_array_sources_copy_into_array_buffer_backed_views() {
        let source = r#"
const shared = new SharedArrayBuffer(8);
const signedSource = new BigInt64Array(shared);
signedSource[0] = 7n;
const signedToSigned = new BigInt64Array(signedSource);
const signedToUnsigned = new BigUint64Array(signedSource);

const unsignedSource = new BigUint64Array(shared);
const unsignedToSigned = new BigInt64Array(unsignedSource);
const unsignedToUnsigned = new BigUint64Array(unsignedSource);

signedToSigned.buffer.constructor === ArrayBuffer && signedToSigned[0] === 7n &&
signedToUnsigned.buffer.constructor === ArrayBuffer && signedToUnsigned[0] === 7n &&
unsignedToSigned.buffer.constructor === ArrayBuffer && unsignedToSigned[0] === 7n &&
unsignedToUnsigned.buffer.constructor === ArrayBuffer && unsignedToUnsigned[0] === 7n;
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("BigInt TypedArray sources should copy into ArrayBuffer-backed views");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_numeric_typed_array_constructor_collects_generator_iterable() {
        let source = r#"
const numbers = new Uint8Array((function* () {
  yield 7;
  yield 42;
})());
[numbers.length, numbers.join(",")].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("numeric TypedArray constructor should collect a generator iterable");
        assert!(
            outcome.note.contains("string(2|7,42)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_bigint_typed_array_constructor_collects_generator_iterable() {
        let source = r#"
const bigints = new BigInt64Array((function* () {
  yield 7n;
  yield 42n;
})());
[bigints.length, bigints.join(",")].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("BigInt TypedArray constructor should collect a generator iterable");
        assert!(
            outcome.note.contains("string(2|7,42)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_numeric_typed_array_constructor_copies_array_argument() {
        let outcome = engine()
            .run_script(
                "const values = new Uint8Array([7, 42]); [values.length, values.join(',')].join('|');",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("numeric TypedArray constructor should copy an array argument");
        assert!(
            outcome.note.contains("string(2|7,42)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_bigint_typed_array_constructor_copies_array_argument() {
        let outcome = engine()
            .run_script(
                "const values = new BigInt64Array([7n, 42n]); [values.length, values.join(',')].join('|');",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("BigInt TypedArray constructor should copy an array argument");
        assert!(
            outcome.note.contains("string(2|7,42)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_bigint_typed_array_constructors_reduce_multi_limb_values_modulo_2_64() {
        let source = r#"
const positive = 18446744073709551618n;
const negative = -340282366920938463463374607431768211458n;
const signedBoundary = 9223372036854775810n;
const signed = new BigInt64Array([positive, negative, signedBoundary]);
const unsigned = new BigUint64Array([positive, negative, signedBoundary]);
[
  signed[0] === 2n,
  signed[1] === -2n,
  signed[2] === -9223372036854775806n
].join(",") + "|" + [
  unsigned[0] === 2n,
  unsigned[1] === 18446744073709551614n,
  unsigned[2] === 9223372036854775810n
].join(",");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("BigInt TypedArray constructors should reduce values modulo 2^64");
        assert!(
            outcome
                .note
                .contains("string(true,true,true|true,true,true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_constructor_uses_modified_array_iterator_next() {
        let source = r#"
const iteratorPrototype = Object.getPrototypeOf([].values());
let remainingValues = [1, 2, 3, 4];
iteratorPrototype.next = function() {
  const done = remainingValues.length === 0;
  const value = remainingValues.pop();
  return { value, done };
};
const values = new Uint8Array([0]);
[values.length, values.join(",")].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray constructor should use a modified Array iterator next method");
        assert!(
            outcome.note.contains("string(4|4,3,2,1)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_constructor_gets_new_target_prototype_before_source_iterator() {
        let source = r#"
const order = [];
const newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, "prototype", {
  get() {
    order.push("prototype");
    return Uint8Array.prototype;
  }
});
const values = {
  get [Symbol.iterator]() {
    order.push("iterator");
    return function() {
      return [7][Symbol.iterator]();
    };
  }
};
const result = Reflect.construct(Uint8Array, [values], newTarget);
order.join(",") + "|" + result.join(",");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect(
                "TypedArray constructor should get newTarget.prototype before inspecting the source iterator",
            );
        assert!(
            outcome.note.contains("string(prototype,iterator|7)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_source_copy_uses_current_resizable_bounds() {
        let source = r#"
const buffer = new ArrayBuffer(4, { maxByteLength: 4 });
const fixed = new Uint8Array(buffer, 0, 4);
const tracking = new Uint8Array(buffer, 0);
tracking[0] = 1;
tracking[1] = 2;
tracking[2] = 3;
tracking[3] = 4;
const offsetCopy = new Uint8Array(new Uint8Array(buffer, 2, 2));

buffer.resize(2);
let fixedThrows = false;
try {
  new Uint8Array(fixed);
} catch (error) {
  fixedThrows = error instanceof TypeError;
}
const shortened = new Uint8Array(tracking);

buffer.resize(0);
const empty = new Uint8Array(tracking);

[
  offsetCopy.byteOffset,
  offsetCopy.join(","),
  fixedThrows,
  shortened.length,
  shortened.join(","),
  empty.length
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray source copies should use current resizable buffer bounds");
        assert!(
            outcome.note.contains("string(0|3,4|true|2|1,2|0)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_repeated_exact_call_contexts_preserve_nested_callback_targets() {
        let source = r#"
function invoke(callback, constructor) {
  callback(constructor);
}

function forEachConstructor(callback) {
  invoke(callback, Int8Array);
  invoke(callback, Uint8Array);
}

let copiedValues = "";
forEachConstructor(function (Source) {
  forEachConstructor(function (Target) {
    const source = new Source(new SharedArrayBuffer(1));
    source[0] = 7;
    const copy = new Target(source);
    copiedValues += copy[0];
  });
});
copiedValues === "7777";
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("repeated exact call contexts should preserve nested callback targets");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_detaches_only_branded_array_buffers_with_private_state() {
        let source = r#"
const buffer = new ArrayBuffer(8);
const view = new DataView(buffer);
const typed = new Uint8Array(buffer);
buffer.$ArrayBufferDataPtr = 0;
buffer.$ArrayBufferByteLength = 99;
let privateBefore = "unset";
try {
  privateBefore = [buffer.byteLength, view.byteLength, typed.length].join(":");
} catch (error) {
  privateBefore = error.name;
}

let forgedTypeError = false;
try {
  __porfDetachArrayBuffer({ $ArrayBufferByteLength: 8 });
} catch (error) {
  forgedTypeError = error instanceof TypeError;
}
let sharedTypeError = false;
try {
  __porfDetachArrayBuffer(new SharedArrayBuffer(8));
} catch (error) {
  sharedTypeError = error instanceof TypeError;
}
let wrongKeyTypeError = false;
try {
  __porfDetachArrayBuffer(buffer, {});
} catch (error) {
  wrongKeyTypeError = error instanceof TypeError;
}

let firstDetach = true;
try { __porfDetachArrayBuffer(buffer); } catch (error) { firstDetach = error.name; }
let viewAccessorTypeError = false;
try { view.byteLength; } catch (error) { viewAccessorTypeError = error instanceof TypeError; }
let viewMethodTypeError = false;
try { view.getUint8(0); } catch (error) { viewMethodTypeError = error instanceof TypeError; }
let detachedState = "unset";
try {
  detachedState = [
    buffer.detached,
    buffer.byteLength,
    buffer.maxByteLength,
    buffer.resizable
  ].join(":");
} catch (error) { detachedState = error.name; }
let typedState = "unset";
try { typedState = [typed.length, typed[0] === undefined].join(":"); }
catch (error) { typedState = error.name; }
let typedToStringTypeError = false;
try { typed.toString(); } catch (error) { typedToStringTypeError = error instanceof TypeError; }
let typedToLocaleStringTypeError = false;
try { typed.toLocaleString(); }
catch (error) { typedToLocaleStringTypeError = error instanceof TypeError; }

buffer.$ArrayBufferDataPtr = 123;
buffer.$ArrayBufferByteLength = 123;
let repeatedDetach = true;
try { __porfDetachArrayBuffer(buffer); } catch (error) { repeatedDetach = error.name; }
let privateAfterRewrite = false;
try { privateAfterRewrite = buffer.detached && buffer.byteLength === 0 && typed.length === 0; }
catch (error) { privateAfterRewrite = error.name; }
const resizableBuffer = new ArrayBuffer(1, { maxByteLength: 2 });
__porfDetachArrayBuffer(resizableBuffer);
const detachedResizableState = [
  resizableBuffer.resizable,
  resizableBuffer.maxByteLength
].join(":");

[
  privateBefore,
  forgedTypeError,
  sharedTypeError,
  wrongKeyTypeError,
  firstDetach,
  detachedState,
  viewAccessorTypeError && viewMethodTypeError,
  typedState,
  typedToStringTypeError && typedToLocaleStringTypeError,
  repeatedDetach,
  privateAfterRewrite,
  detachedResizableState
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("ArrayBuffer detachment should use private state and realm-correct errors");
        assert!(
            outcome.note.contains(
                "string(8:8:8|true|true|true|true|true:0:0:false|true|0:true|true|true|true|true:0)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_detaches_numeric_typed_array_owned_buffer() {
        let outcome = engine()
            .run_script(
                "const view = new Uint8Array(1); const buffer = view.buffer; __porfDetachArrayBuffer(buffer); buffer.detached && buffer.byteLength === 0 && view.length === 0;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("numeric TypedArray owned buffer should detach");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_detaches_bigint_typed_array_owned_buffer() {
        let outcome = engine()
            .run_script(
                "const view = new BigInt64Array(1); const buffer = view.buffer; __porfDetachArrayBuffer(buffer); buffer.detached && buffer.byteLength === 0 && view.length === 0;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("BigInt TypedArray owned buffer should detach");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_growable_shared_array_buffer_grows_in_place_and_updates_live_views() {
        let source = r#"
const buffer = new SharedArrayBuffer(2, { maxByteLength: 8 });
const tracking = new Uint8Array(buffer);
const fixed = new Uint8Array(buffer, 0, 2);
const view = new DataView(buffer);
tracking[0] = 9;
tracking[1] = 7;

const result = buffer.grow(6);
const zeroInitialized = tracking[2] === 0 && tracking[3] === 0 &&
  tracking[4] === 0 && tracking[5] === 0;
tracking[5] = 11;
const sameSize = buffer.grow(6);

let shrinkRangeError = false;
try { buffer.grow(5); } catch (error) {
  shrinkRangeError = error instanceof RangeError;
}
let excessiveRangeError = false;
try { buffer.grow(9); } catch (error) {
  excessiveRangeError = error instanceof RangeError;
}
let argumentCoerced = false;
let fixedTypeError = false;
try {
  new SharedArrayBuffer(1).grow({
    valueOf() { argumentCoerced = true; return 2; }
  });
} catch (error) {
  fixedTypeError = error instanceof TypeError;
}
let inheritedArgumentCoerced = false;
let inheritedTypeError = false;
try {
  SharedArrayBuffer.prototype.grow.call(Object.create(buffer), {
    valueOf() { inheritedArgumentCoerced = true; return 7; }
  });
} catch (error) {
  inheritedTypeError = error instanceof TypeError;
}

const nested = new SharedArrayBuffer(0, { maxByteLength: 8 });
let nestedRangeError = false;
try {
  nested.grow({
    valueOf() { nested.grow(6); return 4; }
  });
} catch (error) {
  nestedRangeError = error instanceof RangeError;
}

const fractional = new SharedArrayBuffer(0, { maxByteLength: 4 });
const negativeFraction = fractional.grow(-0.5);
const positiveFraction = fractional.grow(1.9);

[
  result === undefined,
  buffer.byteLength,
  buffer.maxByteLength,
  buffer.growable,
  tracking.length,
  fixed.length,
  view.byteLength,
  tracking[0],
  tracking[1],
  zeroInitialized,
  tracking[5],
  fixed[0],
  sameSize === undefined,
  shrinkRangeError,
  excessiveRangeError,
  fixedTypeError,
  argumentCoerced,
  inheritedTypeError,
  inheritedArgumentCoerced,
  nestedRangeError,
  nested.byteLength,
  negativeFraction === undefined,
  positiveFraction === undefined,
  fractional.byteLength
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("growable SharedArrayBuffer should update live views without reallocating");
        assert!(
            outcome.note.contains(
                "string(true|6|8|true|6|2|6|9|7|true|11|9|true|true|true|true|false|true|false|true|6|true|true|1)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_shared_array_buffer_max_byte_length_uses_to_index() {
        let source = r#"
const nan = new SharedArrayBuffer(0, { maxByteLength: NaN });
const negativeFraction = new SharedArrayBuffer(0, { maxByteLength: -0.5 });
let coercions = 0;
const coerced = new SharedArrayBuffer(2, {
  maxByteLength: {
    valueOf() { coercions += 1; return 3.9; }
  }
});
let negativeRangeError = false;
try {
  new SharedArrayBuffer(0, { maxByteLength: -1 });
} catch (error) {
  negativeRangeError = error instanceof RangeError;
}
[
  nan.byteLength,
  nan.maxByteLength,
  nan.growable,
  negativeFraction.maxByteLength,
  coerced.byteLength,
  coerced.maxByteLength,
  coercions,
  negativeRangeError
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("SharedArrayBuffer maxByteLength should use ToIndex semantics");
        assert!(
            outcome.note.contains("string(0|0|true|0|2|3|1|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_shared_array_buffer_grow_throws_errors_from_its_function_realm() {
        let source = r#"
const other = __porfCreateRealm().global;
const grow = other.SharedArrayBuffer.prototype.grow;
const byteLength = Object.getOwnPropertyDescriptor(
  other.SharedArrayBuffer.prototype,
  "byteLength"
).get;
const fixed = new SharedArrayBuffer(1);
let typeRealm = "none";
try {
  grow.call(fixed, 2);
} catch (error) {
  typeRealm = [
    Object.getPrototypeOf(error) === other.TypeError.prototype,
    error instanceof other.TypeError,
    error instanceof TypeError
  ].join(":");
}

const growable = new other.SharedArrayBuffer(1, { maxByteLength: 2 });
let rangeRealm = "none";
try {
  growable.grow(0);
} catch (error) {
  rangeRealm = [
    Object.getPrototypeOf(error) === other.RangeError.prototype,
    error instanceof other.RangeError,
    error instanceof RangeError
  ].join(":");
}

let getterRealm = "none";
try {
  byteLength.call(new ArrayBuffer(1));
} catch (error) {
  getterRealm = [
    Object.getPrototypeOf(error) === other.TypeError.prototype,
    error instanceof other.TypeError,
    error instanceof TypeError
  ].join(":");
}
typeRealm + "|" + rangeRealm + "|" + getterRealm;
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("SharedArrayBuffer grow errors should use the method's defining realm");
        assert!(
            outcome
                .note
                .contains("string(true:true:false|true:true:false|true:true:false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_supports_destructured_object_parameter() {
        // Destructured object parameters (the case `wasm_emit_reports_
        // unsupported_slice_precisely` used to lock as unsupported) now
        // compile and run correctly on this branch.
        let outcome = engine()
            .run_script(
                "function f({ x }) { return x; } f({ x: 42 });",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("destructured object parameter should compile and run");
        assert!(outcome.note.contains("number(42"), "note: {}", outcome.note);
    }

    #[test]
    fn wasm_emit_reports_unsupported_slice_precisely() {
        // Destructured object parameters (the case this test previously
        // locked as unsupported) now compile and run correctly on this
        // branch — see `wasm_backend_supports_destructured_object_parameter`
        // above for that positive coverage. Repoint this test at dynamic
        // `eval`, a permanent product invariant ("compile JavaScript
        // directly to Wasm; do not ship interpreter-in-Wasm") rather than a
        // parameter-shape gap that can be closed by future work, so the
        // unsupported-slice error-message format stays locked without going
        // stale again.
        let unit = engine()
            .compile_script("eval(\"1\");", CompileOptions::default())
            .expect("script compile should succeed");
        let err = engine()
            .emit_wasm(&unit)
            .expect_err("unsupported slice should fail");
        assert!(err
            .message()
            .contains("unsupported in porffor wasm-aot first slice"));
    }

    #[test]
    fn run_defaults_to_wasm_aot() {
        // Product invariant: Wasm-AOT is the default execution backend
        // everywhere. Selecting spec-exec (the hidden differential oracle)
        // requires an explicit `RunOptions::backend` override.
        assert_eq!(ExecutionBackend::default(), ExecutionBackend::WasmAot);
        assert_eq!(RunOptions::default().backend, ExecutionBackend::WasmAot);
        let outcome = engine()
            .run_script("1 + 1;", CompileOptions::default(), RunOptions::default())
            .expect("wasm-aot should run a simple script by default");
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    }

    #[test]
    #[cfg(not(feature = "spec-exec-oracle"))]
    fn spec_exec_backend_errors_clearly_when_oracle_feature_is_off() {
        // Product/release builds compile without the `spec-exec-oracle`
        // feature, so selecting the interpreter oracle backend must fail
        // loudly instead of silently falling back to Wasm-AOT or panicking.
        let err = engine()
            .run_script(
                "1 + 1;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::SpecExec,
                    ..RunOptions::default()
                },
            )
            .expect_err("spec-exec must not be usable without the spec-exec-oracle feature");
        assert!(err.message().contains("spec-exec-oracle"));
    }

    #[test]
    #[cfg(feature = "spec-exec-oracle")]
    fn spec_exec_backend_runs_as_developer_oracle_when_feature_enabled() {
        let outcome = engine()
            .run_script(
                "1 + 1;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::SpecExec,
                    ..RunOptions::default()
                },
            )
            .expect("spec exec oracle should run a simple script when explicitly enabled");
        assert_eq!(outcome.backend_used, ExecutionBackend::SpecExec);
    }

    #[test]
    fn wasm_backend_runs_supported_script() {
        let outcome = engine()
            .run_script(
                "let x = 40; const y = 2; x + y;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run supported script");
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(outcome.note.contains("number(42"));
    }

    #[test]
    fn wasm_backend_supports_string_symbol_hook_dispatch() {
        let outcome = engine()
            .run_script(
                r#"
function IsHTMLDDA() { return null; }
Object.defineProperty(IsHTMLDDA, "$IsHTMLDDA", { value: true });
var $262 = { IsHTMLDDA: IsHTMLDDA };
var total = 0;
function invoke(name, target) {
  if (name === "match") return "".match(target);
  if (name === "matchAll") return "".matchAll(target);
  if (name === "replace") return "".replace(target, target);
  if (name === "replaceAll") return "".replaceAll(target, target);
  if (name === "search") return "".search(target);
  if (name === "split") return "".split(target);
  throw name;
}
function check(name, symbol, expectedArgs) {
  var target = $262.IsHTMLDDA;
  var gets = 0;
  Object.defineProperty(target, symbol, {
    get: function() {
      gets += 1;
      return function() {
        if (this !== target) throw "this";
        if (arguments.length !== expectedArgs) throw "argc";
        if (arguments[0] !== "") throw "arg0";
        if ((name === "replace" || name === "replaceAll") && arguments[1] !== target) {
          throw "arg1";
        }
        return null;
      };
    },
    configurable: true
  });
  if (invoke(name, target) !== null) throw name;
  if (gets !== 1) throw "gets";
  total += gets;
  delete target[symbol];
}
check("match", Symbol.match, 1);
check("matchAll", Symbol.matchAll, 1);
check("replace", Symbol.replace, 2);
check("replaceAll", Symbol.replace, 2);
check("search", Symbol.search, 1);
check("split", Symbol.split, 2);
total;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should dispatch String.prototype symbol hooks");
        assert!(outcome.note.contains("number(6"));
    }

    #[test]
    fn wasm_backend_supports_annexb_date_legacy_methods() {
        let outcome = engine()
            .run_script(
                r#"
var total = 0;
if (new Date(1899, 0).getYear() !== -1) throw "getYear 1899";
if (new Date(1970, 0).getYear() !== 70) throw "getYear 1970";
if (new Date({}).getYear() === new Date({}).getYear()) throw "invalid getYear";
var d = new Date(1970, 1, 2, 3, 4, 5);
var expected = new Date(1971, 1, 2, 3, 4, 5).valueOf();
if (d.setYear(71) !== expected) throw "setYear relative";
if (d.valueOf() !== expected) throw "setYear value";
d = new Date(1970, 0);
d.setYear(2000);
if (d.getFullYear() !== 2000) throw "setYear absolute";
d = new Date(0);
if (d.setYear(NaN) === d.setYear(NaN)) throw "setYear NaN";
d = new Date(0);
if (d.setYear() === d.setYear()) throw "setYear undefined";
var threw = 0;
try { Date.prototype.getYear.call({}); } catch (e) { if (e.name === "TypeError") threw += 1; }
try { Date.prototype.setYear.call(null, 1); } catch (e) { if (e.name === "TypeError") threw += 1; }
if (threw !== 2) throw "receiver TypeError";
if (Date.prototype.toGMTString !== Date.prototype.toUTCString) throw "GMT alias";
if (Date.prototype.getYear.length !== 0 || Date.prototype.setYear.length !== 1) throw "length";
if (Date.prototype.getYear.name !== "getYear" || Date.prototype.setYear.name !== "setYear") throw "name";
262;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should support AnnexB Date legacy methods");
        assert!(outcome.note.contains("number(262"));
    }

    #[test]
    fn wasm_backend_supports_date_core_time_values_and_timezone_offset() {
        let outcome = engine()
            .run_script(
                r#"
if (Date.now() !== 0) throw "Date.now deterministic";
if (new Date(6.54321).valueOf() !== 6) throw "positive TimeClip";
if (new Date(-6.54321).valueOf() !== -6) throw "negative TimeClip";
if (new Date(-0).valueOf() !== 0) throw "negative zero TimeClip";
if (1 / new Date(-0).valueOf() !== Infinity) throw "positive zero TimeClip";
if (new Date(Infinity).valueOf() === new Date(Infinity).valueOf()) throw "Infinity TimeClip";
if (new Date(-Infinity).valueOf() === new Date(-Infinity).valueOf()) throw "-Infinity TimeClip";
if (new Date(2016, 0, 1, 0, 0, 0, -1).getFullYear() !== 2015) throw "ms underflow";
if (new Date(2016, 11, 31, 23, 59, 59, 1000).getFullYear() !== 2017) throw "ms overflow";
if (new Date(0).getTimezoneOffset() !== 0) throw "timezone offset";
if (new Date(NaN).getTimezoneOffset() === new Date(NaN).getTimezoneOffset()) throw "invalid timezone offset";
var threw = 0;
try { Date.prototype.getTimezoneOffset.call({}); } catch (e) { if (e.name === "TypeError") threw += 1; }
try { new Date().getTimezoneOffset.prototype; } catch (e) {}
if (threw !== 1) throw "timezone receiver";
if (Date.prototype.getTimezoneOffset.length !== 0) throw "timezone length";
if (Date.prototype.getTimezoneOffset.name !== "getTimezoneOffset") throw "timezone name";
var constructThrew = 0;
try { new Date.prototype.getTimezoneOffset(); } catch (e) { if (e.name === "TypeError") constructThrew = 1; }
if (constructThrew !== 1) throw "timezone construct";
262;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should support Date core time values");
        assert!(outcome.note.contains("number(262"));
    }

    #[test]
    fn wasm_backend_keeps_date_now_body_available_across_control_flow() {
        let outcome = engine()
            .run_script(
                "Date.now(); if (false) 1; if (Date.now() !== 0) throw 'Date.now'; 262;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Date.now should not be replaced by the deferred builtin stub");
        assert!(outcome.note.contains("number(262"));
    }

    #[test]
    fn wasm_backend_supports_date_component_getters() {
        let outcome = engine()
            .run_script(
                r#"
function check(date, year, month, day, weekDay, hour, minute, second, ms, label) {
  if (date.getUTCFullYear() !== year) throw label + " utc year";
  if (date.getUTCMonth() !== month) throw label + " utc month";
  if (date.getUTCDate() !== day) throw label + " utc date";
  if (date.getUTCDay() !== weekDay) throw label + " utc day";
  if (date.getUTCHours() !== hour) throw label + " utc hours";
  if (date.getUTCMinutes() !== minute) throw label + " utc minutes";
  if (date.getUTCSeconds() !== second) throw label + " utc seconds";
  if (date.getUTCMilliseconds() !== ms) throw label + " utc ms";
  if (date.getFullYear() !== year) throw label + " local year";
  if (date.getMonth() !== month) throw label + " local month";
  if (date.getDate() !== day) throw label + " local date";
  if (date.getDay() !== weekDay) throw label + " local day";
  if (date.getHours() !== hour) throw label + " local hours";
  if (date.getMinutes() !== minute) throw label + " local minutes";
  if (date.getSeconds() !== second) throw label + " local seconds";
  if (date.getMilliseconds() !== ms) throw label + " local ms";
}
check(new Date(0), 1970, 0, 1, 4, 0, 0, 0, 0, "epoch");
check(new Date(-1), 1969, 11, 31, 3, 23, 59, 59, 999, "negative");
check(new Date(951868799999), 2000, 1, 29, 2, 23, 59, 59, 999, "leap boundary");
check(new Date(951868800000), 2000, 2, 1, 3, 0, 0, 0, 0, "march boundary");
var invalid = new Date(NaN);
if (invalid.getUTCMonth() === invalid.getUTCMonth()) throw "invalid utc month";
if (invalid.getDate() === invalid.getDate()) throw "invalid local date";
var threw = 0;
try { Date.prototype.getUTCMonth.call({}); } catch (e) { if (e.name === "TypeError") threw += 1; }
try { Date.prototype.getMilliseconds.call(null); } catch (e) { if (e.name === "TypeError") threw += 1; }
if (threw !== 2) throw "receiver TypeError";
function meta(fn, name) {
  if (fn.length !== 0) throw name + " length";
  if (fn.name !== name) throw name + " name";
  var constructThrew = 0;
  try { new fn(); } catch (e) { if (e.name === "TypeError") constructThrew = 1; }
  if (constructThrew !== 1) throw name + " construct";
}
meta(Date.prototype.getUTCFullYear, "getUTCFullYear");
meta(Date.prototype.getUTCMonth, "getUTCMonth");
meta(Date.prototype.getUTCDate, "getUTCDate");
meta(Date.prototype.getUTCDay, "getUTCDay");
meta(Date.prototype.getUTCHours, "getUTCHours");
meta(Date.prototype.getUTCMinutes, "getUTCMinutes");
meta(Date.prototype.getUTCSeconds, "getUTCSeconds");
meta(Date.prototype.getUTCMilliseconds, "getUTCMilliseconds");
meta(Date.prototype.getMonth, "getMonth");
meta(Date.prototype.getDate, "getDate");
meta(Date.prototype.getDay, "getDay");
meta(Date.prototype.getHours, "getHours");
meta(Date.prototype.getMinutes, "getMinutes");
meta(Date.prototype.getSeconds, "getSeconds");
meta(Date.prototype.getMilliseconds, "getMilliseconds");
262;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should support Date component getters");
        assert!(outcome.note.contains("number(262"));
    }

    #[test]
    fn wasm_backend_supports_date_component_setters() {
        let outcome = engine()
            .run_script(
                r#"
function same(a, b, label) {
  if (a.valueOf() !== b.valueOf()) throw label;
}
var d = new Date(0);
if (d.setUTCFullYear(2000, 1, 29) !== 951782400000) throw "setUTCFullYear return";
if (d.getUTCFullYear() !== 2000 || d.getUTCMonth() !== 1 || d.getUTCDate() !== 29) throw "setUTCFullYear value";
d = new Date(0);
if (d.setUTCMonth(12) !== 31536000000) throw "month overflow";
if (d.getUTCFullYear() !== 1971 || d.getUTCMonth() !== 0 || d.getUTCDate() !== 1) throw "month overflow value";
d = new Date(2000, 0, 1);
d.setUTCDate(0);
if (d.getUTCFullYear() !== 1999 || d.getUTCMonth() !== 11 || d.getUTCDate() !== 31) throw "date underflow";
d = new Date(0);
d.setUTCHours(1, 2, 3, 4);
if (d.getUTCHours() !== 1 || d.getUTCMinutes() !== 2 || d.getUTCSeconds() !== 3 || d.getUTCMilliseconds() !== 4) throw "setUTCHours";
d.setUTCMinutes(10);
if (d.getUTCHours() !== 1 || d.getUTCMinutes() !== 10 || d.getUTCSeconds() !== 3 || d.getUTCMilliseconds() !== 4) throw "setUTCMinutes default";
d.setUTCSeconds(60);
if (d.getUTCMinutes() !== 11 || d.getUTCSeconds() !== 0) throw "seconds overflow";
d.setUTCMilliseconds(-1);
if (d.getUTCSeconds() !== 59 || d.getUTCMilliseconds() !== 999) throw "ms underflow";

same(new Date(new Date(0).setFullYear(2001, 2, 4)), new Date(new Date(0).setUTCFullYear(2001, 2, 4)), "local full year");
same(new Date(new Date(0).setMonth(5, 6)), new Date(new Date(0).setUTCMonth(5, 6)), "local month");
same(new Date(new Date(0).setDate(7)), new Date(new Date(0).setUTCDate(7)), "local date");
same(new Date(new Date(0).setHours(8, 9, 10, 11)), new Date(new Date(0).setUTCHours(8, 9, 10, 11)), "local hours");
same(new Date(new Date(0).setMinutes(12, 13, 14)), new Date(new Date(0).setUTCMinutes(12, 13, 14)), "local minutes");
same(new Date(new Date(0).setSeconds(15, 16)), new Date(new Date(0).setUTCSeconds(15, 16)), "local seconds");
same(new Date(new Date(0).setMilliseconds(17)), new Date(new Date(0).setUTCMilliseconds(17)), "local ms");

d = new Date(NaN);
if (d.setUTCFullYear(2000) !== 946684800000) throw "invalid full year return";
if (d.getUTCFullYear() !== 2000 || d.getUTCMonth() !== 0 || d.getUTCDate() !== 1) throw "invalid full year value";
d = new Date(NaN);
if (d.setUTCMonth(0) === d.setUTCMonth(0)) throw "invalid month stays NaN";
if (d.valueOf() === d.valueOf()) throw "invalid month stored";

var order = "";
d = new Date(0);
try {
  Date.prototype.setUTCFullYear.call({}, { valueOf: function() { order += "x"; return 1; } });
} catch (e) { if (e.name === "TypeError") order += "t"; }
if (order !== "t") throw "receiver before coercion";

d = new Date(2000, 0, 31);
d.setUTCMonth({ valueOf: function() { d.setUTCDate(1); return 1; } });
if (d.getUTCFullYear() !== 2000 || d.getUTCMonth() !== 2 || d.getUTCDate() !== 2) throw "old value defaults";

order = "";
d = new Date(0);
d.setUTCHours(
  { valueOf: function() { order += "h"; return 1; } },
  { valueOf: function() { order += "m"; return 2; } },
  { valueOf: function() { order += "s"; return 3; } },
  { valueOf: function() { order += "n"; return 4; } }
);
if (order !== "hmsn") throw "coercion order";

var threw = 0;
try { Date.prototype.setUTCDate.call(null, 1); } catch (e) { if (e.name === "TypeError") threw += 1; }
try { new Date.prototype.setUTCSeconds(); } catch (e) { if (e.name === "TypeError") threw += 1; }
if (threw !== 2) throw "errors";
function meta(fn, name, length) {
  if (fn.length !== length) throw name + " length";
  if (fn.name !== name) throw name + " name";
  var desc = Object.getOwnPropertyDescriptor(Date.prototype, name);
  if (desc.enumerable || !desc.writable || !desc.configurable) throw name + " descriptor";
}
meta(Date.prototype.setUTCFullYear, "setUTCFullYear", 3);
meta(Date.prototype.setUTCMonth, "setUTCMonth", 2);
meta(Date.prototype.setUTCDate, "setUTCDate", 1);
meta(Date.prototype.setUTCHours, "setUTCHours", 4);
meta(Date.prototype.setUTCMinutes, "setUTCMinutes", 3);
meta(Date.prototype.setUTCSeconds, "setUTCSeconds", 2);
meta(Date.prototype.setUTCMilliseconds, "setUTCMilliseconds", 1);
meta(Date.prototype.setFullYear, "setFullYear", 3);
meta(Date.prototype.setMonth, "setMonth", 2);
meta(Date.prototype.setDate, "setDate", 1);
meta(Date.prototype.setHours, "setHours", 4);
meta(Date.prototype.setMinutes, "setMinutes", 3);
meta(Date.prototype.setSeconds, "setSeconds", 2);
meta(Date.prototype.setMilliseconds, "setMilliseconds", 1);
262;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should support Date component setters");
        assert!(outcome.note.contains("number(262"));
    }

    #[test]
    fn wasm_backend_supports_date_utc_and_decimal_exponent_to_number() {
        let outcome = engine()
            .run_script(
                r#"
function same(actual, expected, label) {
  if (actual !== expected) throw label;
}
same(Date.UTC(1970), 0, "year only");
same(Date.UTC(1970, 0, 1, 0, 0, 0, 1), 1, "full args");
same(Date.UTC(1970, 12, 1), 31536000000, "month overflow");
same(Date.UTC(1970, 0, 0), -86400000, "date underflow");
same(Date.UTC(0, 0, 1), -2208988800000, "year remap");
if (Date.UTC() === Date.UTC()) throw "missing year NaN";
if (Date.UTC(1970, NaN) === Date.UTC(1970, NaN)) throw "NaN arg";
same(Date.UTC(275760, 8, 13), 8640000000000000, "clip max");
if (Date.UTC(275760, 8, 14) === Date.UTC(275760, 8, 14)) throw "clip overflow";
var order = "";
Date.UTC({ valueOf: function() { order += "y"; return 1970; } }, { valueOf: function() { order += "m"; return 0; } });
if (order !== "ym") throw "coercion order";
var symbolThrew = 0;
try { Date.UTC(Symbol("x")); } catch (e) { symbolThrew = 1; }
if (symbolThrew !== 1) throw "symbol";
if (Date.UTC.length !== 7 || Date.UTC.name !== "UTC") throw "metadata";
var desc = Object.getOwnPropertyDescriptor(Date, "UTC");
if (desc.enumerable || !desc.writable || !desc.configurable) throw "descriptor";
var constructThrew = 0;
try { new Date.UTC(); } catch (e) { constructThrew = 1; }
if (constructThrew !== 1) throw "construct";

same(Number("   +00200.000E-0002\t"), 2, "trimmed exponent");
same(Number("1e3"), 1000, "positive exponent");
same(Number("1E-3"), 0.001, "negative exponent");
if (Number("not a number") === Number("not a number")) throw "malformed text";
if (Number("1e") === Number("1e")) throw "missing exponent digits";
if (Number("1e+") === Number("1e+")) throw "missing signed exponent digits";
if (Number("1e-") === Number("1e-")) throw "missing negative exponent digits";
if (Number("e1") === Number("e1")) throw "missing significand";
var d = new Date(0);
same(d.setTime("   +00200.000E-0002\t"), 2, "setTime exponent");
d = new Date(0);
same(d.setUTCMilliseconds("   +00200.000E-0002\t"), 2, "setter exponent");
d = new Date(0);
if (d.setYear("not a number") === d.setYear("not a number")) throw "setYear malformed return";
if (d.valueOf() === d.valueOf()) throw "setYear malformed stores NaN";
262;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should support Date.UTC and decimal exponent ToNumber");
        assert!(outcome.note.contains("number(262"));
    }

    #[test]
    fn wasm_backend_supports_remainder() {
        let outcome = engine()
            .run_script(
                "7 % 3;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run remainder");
        assert!(outcome.note.contains("number(1"));
    }

    #[test]
    fn wasm_backend_supports_assignment_and_if() {
        let outcome = engine()
            .run_script(
                "let x = 0; if (!x) { x = 5; } x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run assignment and if");
        assert!(outcome.note.contains("number(5"));
    }

    #[test]
    fn wasm_backend_rejects_const_assignment_precisely() {
        let err = engine()
            .run_script(
                "const x = 1; x = 2;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("const assignment should stay unsupported");
        assert!(err
            .message()
            .contains("unsupported in porffor wasm-aot first slice: assignment to const binding"));
    }

    #[test]
    fn wasm_backend_supports_hoisted_function_calls() {
        let outcome = engine()
            .run_script(
                "add(1, 2); function add(x, y) { return x + y; }",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run hoisted function call");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_coercive_subtraction_preserves_dynamic_numeric_kind() {
        let outcome = engine()
            .run_script(
                "function subtract(left, right) { return left - right; } subtract(7, 2) === 5 && subtract(7n, 2n) === 5n;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should subtract dynamic Number and BigInt arguments");
        assert!(
            outcome.note.contains("boolean(true"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_supports_direct_recursion() {
        let outcome = engine()
            .run_script(
                "function up(n) { if (n === 0) { return 0; } return up(n - 1) + 1; } up(3);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run recursion");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_object_property_read() {
        let outcome = engine()
            .run_script(
                "let o = { x: 1, y: 2 }; o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run object property read");
        assert!(outcome.note.contains("number(1"));
    }

    #[test]
    fn wasm_backend_supports_array_write_and_read() {
        let outcome = engine()
            .run_script(
                "let a = [1]; a[2] = 4; a[2];",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run array write and read");
        assert!(outcome.note.contains("number(4"));
    }

    #[test]
    fn wasm_backend_supports_high_sparse_array_indexes() {
        let outcome = engine()
            .run_script(
                "let a = [1]; a[50000] = 2; let b = a.map(function(v) { return v + 1; }); b.length + \"|\" + b[0] + \"|\" + b[50000];",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should preserve high sparse array indexes");
        assert!(
            outcome.note.contains("string(50001|2|3)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_supports_missing_heap_reads() {
        let object_outcome = engine()
            .run_script(
                "let o = {}; o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should return undefined for missing property");
        assert!(object_outcome.note.contains("undefined("));

        let array_outcome = engine()
            .run_script(
                "let a = [1]; a[9];",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should return undefined for missing index");
        assert!(array_outcome.note.contains("undefined("));
    }

    #[test]
    fn wasm_backend_supports_object_bracket_write() {
        let outcome = engine()
            .run_script(
                "let o = { x: 1 }; o[\"x\"] = 3; o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should support object bracket write");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_object_return_from_function() {
        let outcome = engine()
            .run_script(
                "function box(x) { let o = { x: x }; return o; } let o = box(2); o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run object-return function");
        assert!(outcome.note.contains("number(2"));
    }

    #[test]
    fn wasm_backend_supports_chained_object_access() {
        let outcome = engine()
            .run_script(
                "let o = { inner: { x: 1 } }; o.inner.x = 4; o.inner.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run chained object access");
        assert!(outcome.note.contains("number(4"));
    }

    #[test]
    fn wasm_backend_supports_call_result_property_and_array_length() {
        let property_outcome = engine()
            .run_script(
                "function box(x) { let o = { x: x }; return o; } box(2).x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run call-result property read");
        assert!(property_outcome.note.contains("number(2"));

        let array_outcome = engine()
            .run_script(
                "let a = [1, 2, 3]; a.length;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run array length read");
        assert!(array_outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_nested_array_object_reads() {
        let outcome = engine()
            .run_script(
                "function make() { let o = { items: [{ x: 1 }, { x: 3 }] }; return o; } make().items[1].x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run nested heap reads");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_function_values_and_indirect_calls() {
        let alias_outcome = engine()
            .run_script(
                "function inc(x) { return x + 1; } let g = inc; g(2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run indirect call through alias");
        assert!(alias_outcome.note.contains("number(3"));

        let passthrough_outcome = engine()
            .run_script(
                "function inc(x) { return x + 1; } function pick() { return inc; } let g = pick(); g(2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run function pass-through");
        assert!(passthrough_outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_method_calls_and_this() {
        let method_outcome = engine()
            .run_script(
                "function inc(x) { return x + 1; } let o = { f: inc }; o.f(2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run method call");
        assert!(method_outcome.note.contains("number(3"));

        let bracket_outcome = engine()
            .run_script(
                "function inc(x) { return x + 1; } let o = { f: inc }; o[\"f\"](2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run bracket method call");
        assert!(bracket_outcome.note.contains("number(3"));

        let this_outcome = engine()
            .run_script(
                "function getX() { return this.x; } let o = { x: 3, f: getX }; o.f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run method call with this");
        assert!(this_outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_evaluates_compound_method_receiver_once() {
        let source = r#"
            let baseCalls = 0;
            function makeReceiver() {
                baseCalls += 1;
                return {
                    value: baseCalls,
                    method: function () { return this.value; }
                };
            }
            let result = makeReceiver().method();
            baseCalls === 1 && result === 1;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should evaluate a compound method receiver once");
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_evaluates_method_base_key_and_arguments_in_order_once() {
        let source = r#"
            let order = "";
            let receiver;
            function makeReceiver() {
                order += "base";
                receiver = {
                    method: function (value) {
                        order += "call";
                        return this === receiver && value === "argument";
                    }
                };
                return receiver;
            }
            function methodKey() {
                order += ",key";
                return "method";
            }
            function argument() {
                order += ",argument";
                return "argument";
            }
            let result = makeReceiver()[methodKey()](argument());
            result && order === "base,key,argumentcall";
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should preserve computed method call evaluation order");
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_evaluates_no_method_arguments_after_abrupt_computed_key() {
        let source = r#"
            let order = "";
            let expected = new Error("key");
            function makeReceiver() {
                order += "base";
                return { method: function () {} };
            }
            function methodKey() {
                order += ",key";
                throw expected;
            }
            function argument() {
                order += ",argument";
            }
            let caught;
            try {
                makeReceiver()[methodKey()](argument());
            } catch (error) {
                caught = error;
            }
            caught === expected && order === "base,key";
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should stop a method call after an abrupt computed key");
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_supports_array_function_calls_and_global_default_this() {
        let array_outcome = engine()
            .run_script(
                "function inc(x) { return x + 1; } let a = [inc]; a[0](2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run array function call");
        assert!(array_outcome.note.contains("number(3"));

        let bare_this_outcome = engine()
            .run_script(
                "function check() { return this; } check();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should default bare-call this to global object");
        assert!(bare_this_outcome.note.contains("object(handle@"));
    }

    #[test]
    fn wasm_backend_grows_heap_beyond_old_fixed_cap() {
        let source = format!(
            "let o = {{}}; {} o.k64;",
            (0..65)
                .map(|index| format!("o[\"k{index}\"] = {index};"))
                .collect::<Vec<_>>()
                .join(" ")
        );
        let outcome = engine()
            .run_script(
                &source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should grow heap");
        assert!(outcome.note.contains("number(64"));
    }

    #[test]
    fn wasm_backend_lowers_property_access_on_dynamic_target() {
        let outcome = engine()
            .run_script(
                "let v; if (true) { v = 1; } else { v = { x: 1 }; } v.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("dynamic property access should run");
        assert!(outcome.note.contains("undefined(undefined)"));
    }

    #[test]
    fn wasm_backend_proxy_get_missing_trap_falls_back_to_object_target() {
        for (source, expected) in [
            (
                "let p = new Proxy({ attr: 1 }, { get: undefined }); p.attr;",
                "number(1)",
            ),
            (
                "let p = new Proxy({ attr: 1 }, { get: undefined }); p.foo;",
                "undefined(undefined)",
            ),
            (
                "let target = { get attr() { return this; } }; let p = Object.create(new Proxy(target, {})); p.attr === p;",
                "boolean(true)",
            ),
            (
                "let a = [1, 2, 3]; let p = new Proxy(new Proxy(a, {}), { get: undefined }); p.length;",
                "number(3)",
            ),
            (
                "let a = [1, 2, 3]; let p = new Proxy(new Proxy(a, {}), { get: undefined }); p[1];",
                "number(2)",
            ),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| {
                    panic!("proxy get missing-trap case should run for `{source}`: {err:?}")
                });
            assert!(
                outcome.note.contains(expected),
                "source: {source}, note: {}",
                outcome.note
            );
        }
    }

    #[test]
    fn wasm_backend_proxy_get_non_callable_trap_throws_type_error() {
        let err = engine()
            .run_script(
                "let p = new Proxy({}, { get: {} }); p.attr;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("non-callable proxy get trap should throw");
        assert!(err.message().contains("uncaught throw: TypeError"));
    }

    #[test]
    fn wasm_backend_proxy_get_trap_receives_well_known_symbol_key() {
        let outcome = engine()
            .run_script(
                r#"
let seen = "";
let target = { next: function() { return { done: true }; } };
let p = new Proxy(target, {
  get: function(target, key, receiver) {
    if (String(key) === "Symbol(Symbol.iterator)") {
      seen = (typeof key) + ":" + (key === Symbol.iterator);
    }
    return Reflect.get(target, key, receiver);
  }
});
Iterator.from(p);
seen;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("proxy get trap should receive Symbol.iterator as a symbol");
        assert!(
            outcome.note.contains("string(symbol:true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_iterator_from_iterable_primitives() {
        let outcome = engine()
            .run_script(
                r#"
Number.prototype[Symbol.iterator] = function* () {
  let i = 0;
  let target = this >>> 0;
  while (i < target) {
    yield i;
    ++i;
  }
};

let primitiveThrows = "no";
try {
  Iterator.from(5);
} catch (e) {
  primitiveThrows = e instanceof TypeError ? "typeerror" : e.name;
}

function numberArraySnapshot(array) {
  return array.length + ":" + array[0] + "," + array[1] + "," + array[2] + "," + array[3] + "," + array[4];
}

let numberArray = numberArraySnapshot(Array.from(5));
let boxedNumberArray = numberArraySnapshot(Array.from(Iterator.from(new Number(5))));
let stringResult = Array.from(Iterator.from("string"));
let stringArray = stringResult.length + ":" + stringResult[0] + stringResult[1] + stringResult[2] + stringResult[3] + stringResult[4] + stringResult[5];

const originalStringIterator = String.prototype[Symbol.iterator];
let observedType;
Object.defineProperty(String.prototype, Symbol.iterator, {
  get() {
    "use strict";
    observedType = typeof this;
    return originalStringIterator;
  },
});
Iterator.from("");
let firstObservedType = observedType;
Iterator.from(new String(""));
let secondObservedType = observedType;

numberArray + "|" + primitiveThrows + "|" + boxedNumberArray + "|" + stringArray + "|" + firstObservedType + "," + secondObservedType;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Iterator.from primitive iterable cases should run");
        assert!(
            outcome
                .note
                .contains("string(5:0,1,2,3,4|typeerror|5:0,1,2,3,4|6:string|string,object)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_symbol_iterator_next() {
        let outcome = engine()
            .run_script(
                r#"
let array = new Int8Array([3, 1, 2]);
let iterator = array[Symbol.iterator]();
let first = iterator.next();
let second = iterator.next();
let third = iterator.next();
let exhausted = iterator.next();
let repeated = iterator.next();
let clamped = new Uint8ClampedArray([255, 1, 2]);
let clampedIterator = clamped[Symbol.iterator]();
let clampedFirst = clampedIterator.next();
first.value + ":" + first.done + "|" +
second.value + ":" + second.done + "|" +
third.value + ":" + third.done + "|" +
typeof exhausted.value + ":" + exhausted.done + "|" +
typeof repeated.value + ":" + repeated.done + "|" +
clampedFirst.value + ":" + clampedFirst.done;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("typed array Symbol.iterator next should run");
        assert!(
            outcome.note.contains(
                "string(3:false|1:false|2:false|undefined:true|undefined:true|255:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_arguments_iterators_observe_length_truncation() {
        let outcome = engine()
            .run_script(
                r#"
function mapped(a, b, c) {
  let iterator = arguments[Symbol.iterator]();
  iterator.next();
  iterator.next();
  arguments.length = 2;
  let result = iterator.next();
  return typeof result.value + ":" + result.done;
}

function unmapped(a, b, c) {
  "use strict";
  let iterator = arguments[Symbol.iterator]();
  iterator.next();
  iterator.next();
  arguments.length = 2;
  let result = iterator.next();
  return typeof result.value + ":" + result.done;
}

mapped(2, 1, 3) + "|" + unmapped(2, 1, 3);
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("arguments iterators should observe length truncation");
        assert!(
            outcome
                .note
                .contains("string(undefined:true|undefined:true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_string_iterator_preserves_code_points_and_brand() {
        let outcome = engine()
            .run_script(
                r#"
let pair = String.fromCharCode(0xD834, 0xDF06);
let iterator = ("a" + pair + "b")[Symbol.iterator]();
let first = iterator.next();
let second = iterator.next();
let third = iterator.next();
let exhausted = iterator.next();
let prototype = Object.getPrototypeOf(iterator);
let other = __porfCreateRealm().global;
let otherIterator = other.String.prototype[Symbol.iterator].call("x");
let otherPrototype = Object.getPrototypeOf(otherIterator);
let realmLocal = otherPrototype !== prototype &&
  Object.getPrototypeOf(otherPrototype) !== Object.getPrototypeOf(prototype);
let marker = {};
let propagated = false;
try {
  String.prototype[Symbol.iterator].call({ toString() { throw marker; } });
} catch (error) {
  propagated = error === marker;
}
first.value + ":" + first.done + "|" +
second.value.length + ":" + (second.value === pair) + "|" +
third.value + ":" + third.done + "|" +
typeof exhausted.value + ":" + exhausted.done + "|" +
prototype[Symbol.toStringTag] + "|" + propagated + "|" + realmLocal;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("String iterator code point and brand cases should run");
        assert!(
            outcome.note.contains(
                "string(a:false|2:true|b:false|undefined:true|String Iterator|true|true)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_push_is_generic_for_objects_and_boxed_booleans() {
        let outcome = engine()
            .run_script(
                r#"
var obj = {};
obj.push = Array.prototype.push;
var firstLen = obj.push(-1);
var first = firstLen + ":" + obj.length + ":" + obj["0"];

obj.length = null;
var nullLen = obj.push(-7);
var afterNull = nullLen + ":" + obj.length + ":" + obj["0"];

obj.length = 4294967296;
var largeLen = obj.push("x", "y");
var afterLarge =
  largeLen + ":" + obj.length + ":" + obj["4294967296"] + ":" + obj["4294967297"] + ":" + (obj["0"] === -7);

var boolLen = Array.prototype.push.call(true);
var stringThrow = "no";
try {
  Array.prototype.push.call("");
} catch (e) {
  stringThrow = e.name;
}
first + "|" + afterNull + "|" + afterLarge + "|" + boolLen + "|" + stringThrow;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.push generic object cases should run");
        assert!(
            outcome
                .note
                .contains("string(1:1:-1|1:1:-7|4294967298:4294967298:x:y:true|0|TypeError)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_push_respects_non_writable_array_length() {
        let outcome = engine()
            .run_script(
                r#"
var zero = [];
Object.defineProperty(zero, "length", { writable: false });
var zeroThrow = "no";
try {
  zero.push();
} catch (e) {
  zeroThrow = e.name;
}

var one = [];
Object.defineProperty(one, "length", { writable: false });
var oneThrow = "no";
try {
  one.push(1);
} catch (e) {
  oneThrow = e.name;
}

zeroThrow + ":" + zero.length + "|" + oneThrow + ":" + one.length + ":" + (one[0] === undefined);
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.push should catch non-writable length TypeErrors");
        assert!(
            outcome
                .note
                .contains("string(TypeError:0|TypeError:0:true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_push_preserves_max_array_length_boundary_property() {
        let outcome = engine()
            .run_script(
                r#"
var array = [];
array.length = 4294967295;
var first = array.push();
var caught = "no";
try {
  array.push("x");
} catch (e) {
  caught = e.name + ":" + (e instanceof RangeError);
}
caught + "|" + first + "|" + array.length + "|" + array[4294967295] + "|" + array["4294967295"];
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.push should store the boundary property before RangeError");
        assert!(
            outcome
                .note
                .contains("string(RangeError:true|4294967295|4294967295|x|x)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_push_calls_inherited_index_setter_before_length_write() {
        let outcome = engine()
            .run_script(
                r#"
var array = [];
var callCount = 0;
var receiverMatches = false;
var receivedValue = 0;
Object.defineProperty(Array.prototype, "0", {
  set: function(value) {
    callCount += 1;
    receiverMatches = this === array;
    receivedValue = value;
    Object.defineProperty(array, "length", { writable: false });
  },
  configurable: true
});

var caught = "no";
try {
  array.push(41);
} catch (e) {
  caught = e.name;
}
delete Array.prototype[0];
caught + ":" + array.length + ":" + callCount + ":" + receiverMatches + ":" +
  receivedValue + ":" + (array[0] === undefined);
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.push should honor inherited index setters");
        assert!(
            outcome.note.contains("string(TypeError:0:1:true:41:true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_assignment_calls_inherited_numeric_setter() {
        let outcome = engine()
            .run_script(
                r#"
var array = [];
var callCount = 0;
var receiverMatches = false;
var receivedValue = 0;
Object.defineProperty(Array.prototype, "0", {
  set: function(value) {
    callCount += 1;
    receiverMatches = this === array;
    receivedValue = value;
  },
  configurable: true
});

var assignedValue = array[0] = 23;
delete Array.prototype[0];
assignedValue + ":" + callCount + ":" + receiverMatches + ":" + receivedValue + ":" +
  array.length + ":" + (array[0] === undefined);
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("array assignment should honor inherited numeric setters");
        assert!(
            outcome.note.contains("string(23:1:true:23:0:true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_reflect_set_calls_inherited_array_numeric_setter() {
        let outcome = engine()
            .run_script(
                r#"
var array = [];
var callCount = 0;
var receiverMatches = false;
Object.defineProperty(Array.prototype, "0", {
  set: function(value) {
    callCount += value;
    receiverMatches = this === array;
  },
  configurable: true
});

var result = Reflect.set(array, "0", 5);
delete Array.prototype[0];
result + ":" + callCount + ":" + receiverMatches + ":" + array.length + ":" +
  (array[0] === undefined);
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Reflect.set should honor inherited array numeric setters");
        assert!(
            outcome.note.contains("string(true:5:true:0:true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_reflect_set_propagates_accessor_exceptions() {
        let outcome = engine()
            .run_script(
                r#"
var target = {};
var receiver = {};
var token = {};
var caught;
var callCount = 0;
Object.defineProperty(target, "slot", {
  set: function() {
    callCount += 1;
    throw token;
  }
});
try {
  Reflect.set(target, "slot", 1, receiver);
} catch (error) {
  caught = error;
}
callCount === 1 && caught === token;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Reflect.set should propagate accessor exceptions");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_push_calls_inherited_setter_with_non_writable_length() {
        let outcome = engine()
            .run_script(
                r#"
var array = [];
Object.defineProperty(array, "length", { writable: false });
var callCount = 0;
Object.defineProperty(Array.prototype, "0", {
  set: function(value) {
    callCount += value;
  },
  configurable: true
});

var caught = "no";
try {
  array.push(2);
} catch (e) {
  caught = e.name;
}
delete Array.prototype[0];
caught + ":" + callCount + ":" + array.length + ":" + (array[0] === undefined);
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("push should run an inherited setter before its final length write");
        assert!(
            outcome.note.contains("string(TypeError:2:0:true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_push_respects_frozen_array_length() {
        let outcome = engine()
            .run_script(
                r#"
var empty = [];
Object.freeze(empty);
var emptyThrow = "no";
try {
  empty.push();
} catch (e) {
  emptyThrow = e.name;
}

var array = [];
var callCount = 0;
Object.defineProperty(Array.prototype, "0", {
  set: function(value) {
    Object.freeze(array);
    callCount += 1;
  },
  configurable: true
});

var inheritedThrow = "no";
try {
  array.push(1);
} catch (e) {
  inheritedThrow = e.name;
}
delete Array.prototype[0];
emptyThrow + ":" + empty.length + ":" + (empty[0] === undefined) + "|" +
  inheritedThrow + ":" + array.length + ":" + callCount + ":" + (array[0] === undefined);
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.push should respect Object.freeze length integrity");
        assert!(
            outcome
                .note
                .contains("string(TypeError:0:true|TypeError:0:1:true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_shift_moves_generic_array_like_properties_and_holes() {
        let outcome = engine()
            .run_script(
                r#"
var sparse = ["a", , "c"];
var sparseFirst = sparse.shift();
var sparseResult = sparseFirst + ":" + sparse.length + ":" + (0 in sparse) + ":" + sparse[1];

var calls = [];
var object;
var prototype = {};
Object.defineProperty(prototype, "0", {
  get: function() {
    calls.push("get 0");
    return "first";
  },
  set: function(value) {
    calls.push("set 0 " + value + " " + (this === object));
  },
  configurable: true
});
Object.defineProperty(prototype, "1", {
  get: function() {
    calls.push("get 1");
    return "second";
  },
  configurable: true
});
object = Object.create(prototype);
object.length = 2;
object.shift = Array.prototype.shift;
var objectFirst = object.shift();
var objectResult = objectFirst + ":" + object.length + ":" + calls.join(",");

var savedShift = Array.prototype.shift;
Array.prototype.shift = function() { return "custom " + this.length; };
var mutated = [9];
var mutatedResult = mutated.shift() + ":" + mutated.length;
Array.prototype.shift = savedShift;
var extracted = [8];
var extractedResult = savedShift.call(extracted) + ":" + extracted.length;

sparseResult + "|" + objectResult + "|" + mutatedResult + "|" + extractedResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.shift generic array-like cases should run");
        assert!(
            outcome.note.contains(
                "string(a:2:false:c|first:1:get 0,get 1,set 0 second true|custom 1:1|8:0)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_shift_observes_proxy_operations_in_spec_order() {
        let outcome = engine()
            .run_script(
                r#"
var calls = [];
var target = { 0: "a", 1: "b", length: 2 };
var proxy = new Proxy(target, {
  get: function(target, key, receiver) {
    calls.push("get " + key);
    return Reflect.get(target, key, receiver);
  },
  has: function(target, key) {
    calls.push("has " + key);
    return Reflect.has(target, key);
  },
  set: function(target, key, value, receiver) {
    calls.push("set " + key + " " + value);
    return Reflect.set(target, key, value, receiver);
  },
  deleteProperty: function(target, key) {
    calls.push("delete " + key);
    return Reflect.deleteProperty(target, key);
  }
});
var first = Array.prototype.shift.call(proxy);
var proxyResult = first + ":" + target.length + ":" + target[0] + ":" + (1 in target) + "|" + calls.join(",");

var boundaryCalls = [];
var boundary = new Proxy({}, {
  get: function(target, key) {
    boundaryCalls.push("get " + key);
    if (key === "length") return 9007199254740991;
    if (key === "0") return "first";
  },
  has: function(target, key) {
    throw key;
  }
});
var boundaryResult = "missing";
try {
  Array.prototype.shift.call(boundary);
} catch (error) {
  boundaryResult = error + ":" + boundaryCalls.join(",");
}

proxyResult + "|" + boundaryResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.shift should use observable object operations in spec order");
        assert!(
            outcome.note.contains(
                "string(a:1:b:false|get length,get 0,has 1,get 1,set 0 b,delete 1,set length 1|1:get length,get 0)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_shift_enforces_strict_writes_and_builtin_realm_errors() {
        let outcome = engine()
            .run_script(
                r#"
var frozen = [];
Object.freeze(frozen);
var frozenResult = "missing";
try {
  frozen.shift();
} catch (error) {
  frozenResult = error.name + ":" + frozen.length;
}

var stringResults = [];
for (var value of ["", "abc"]) {
  try {
    Array.prototype.shift.call(value);
    stringResults.push("missing");
  } catch (error) {
    stringResults.push(error.name);
  }
}

var other = __porfCreateRealm();
var otherShift = other.global.Array.prototype.shift;
var realmResult = "missing";
try {
  otherShift.call(null);
} catch (error) {
  realmResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

frozenResult + "|" + stringResults.join(",") + "|" + realmResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.shift should enforce strict writes in its defining realm");
        assert!(
            outcome
                .note
                .contains("string(TypeError:0|TypeError,TypeError|true:true:false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_unshift_moves_generic_array_like_properties_and_holes() {
        let outcome = engine()
            .run_script(
                r#"
var object = { 0: "a", 2: "c", length: 3 };
var objectLength = Array.prototype.unshift.call(object, "x", "y");
var objectResult = objectLength + ":" + object.length + ":" + object[0] + ":" +
  object[1] + ":" + object[2] + ":" + (3 in object) + ":" + object[4];

var array = [];
array[1] = "b";
array.length = 3;
var arrayLength = array.unshift("x", "y");
var arrayResult = arrayLength + ":" + array.length + ":" + array[0] + ":" +
  array[1] + ":" + (2 in array) + ":" + array[3] + ":" + (4 in array);

var many = ["tail"];
var manyLength = many.unshift(0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
var manyResult = manyLength + ":" + many.length + ":" + many[0] + ":" +
  many[8] + ":" + many[9] + ":" + many[10];

objectResult + "|" + arrayResult + "|" + manyResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.unshift generic array-like cases should run");
        assert!(
            outcome
                .note
                .contains("string(5:5:x:y:a:false:c|5:5:x:y:false:b:false|11:11:0:8:9:tail)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_unshift_propagates_an_abrupt_source_getter() {
        let error = engine()
            .run_script(
                r#"
var object = { length: 2 };
Object.defineProperty(object, "1", {
  get: function() {
    return "moved";
  },
  configurable: true
});
Object.defineProperty(object, "0", {
  get: function() {
    throw "stop";
  },
  configurable: true
});
Array.prototype.unshift.call(object, "new");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("Array.prototype.unshift should propagate abrupt source getters");
        assert!(
            error.message().contains("uncaught throw") && error.message().contains("string(stop)"),
            "error: {error:?}"
        );
    }

    #[test]
    fn wasm_backend_array_unshift_calls_inherited_setter_before_length_write() {
        let outcome = engine()
            .run_script(
                r#"
var array = [];
var callCount = 0;
var receiverMatches = false;
var receivedValue = 0;
Object.defineProperty(Array.prototype, "0", {
  set: function(value) {
    callCount += 1;
    receiverMatches = this === array;
    receivedValue = value;
  },
  configurable: true
});
Object.defineProperty(array, "length", { writable: false });

var nonzeroThrow = "no";
try {
  array.unshift(41);
} catch (error) {
  nonzeroThrow = error.name;
}

var frozen = [];
Object.freeze(frozen);
var zeroThrow = "no";
try {
  frozen.unshift();
} catch (error) {
  zeroThrow = error.name;
}
delete Array.prototype[0];
nonzeroThrow + ":" + array.length + ":" + callCount + ":" + receiverMatches + ":" +
  receivedValue + ":" + (array[0] === undefined) + "|" + zeroThrow + ":" + frozen.length;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.unshift should honor setters and length integrity");
        assert!(
            outcome
                .note
                .contains("string(TypeError:0:1:true:41:true|TypeError:0)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_proxy_revocable_get_after_revoke_throws_type_error() {
        let err = engine()
            .run_script(
                "let p = Proxy.revocable({}, {}); p.revoke(); p.proxy.attr;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("revoked proxy get should throw");
        assert!(err.message().contains("uncaught throw: TypeError"));
    }

    #[test]
    fn wasm_backend_create_realm_exposes_proxy_constructor() {
        let outcome = engine()
            .run_script(
                r#"
let OProxy = __porfCreateRealm().global.Proxy;
let p = new OProxy({ attr: 7 }, {});
(typeof OProxy) + ":" + (OProxy === Proxy) + ":" + (p.attr === 7);
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm should expose Proxy constructor");
        assert!(
            outcome.note.contains("string(function:false:true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_iterator_constructor_call_throws_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let C = other.Iterator;
let iteratorPrototype = C.prototype;
let constructorDesc = Object.getOwnPropertyDescriptor(iteratorPrototype, "constructor");
let mainConstructorDesc = Object.getOwnPropertyDescriptor(Iterator.prototype, "constructor");
let tagDesc = Object.getOwnPropertyDescriptor(iteratorPrototype, Symbol.toStringTag);
let mainTagDesc = Object.getOwnPropertyDescriptor(Iterator.prototype, Symbol.toStringTag);
let helperNames = [
  "toArray", "forEach", "every", "some", "find", "reduce",
  "map", "filter", "flatMap", "take", "drop"
];
let helperResults = [];
let thrown = "none";
let toArrayThrow = "none";
let disposeThrow = "none";
for (let i = 0; i < helperNames.length; i++) {
  let name = helperNames[i];
  helperResults.push(
    name + ":" +
    (typeof iteratorPrototype[name]) + ":" +
    (iteratorPrototype[name] === Iterator.prototype[name])
  );
}
try {
  C();
} catch (error) {
  thrown = [
    Object.getPrototypeOf(error) === other.TypeError.prototype,
    error instanceof other.TypeError,
    error instanceof TypeError
  ].join(":");
}
try {
  iteratorPrototype.toArray.call(null);
} catch (error) {
  toArrayThrow = [
    Object.getPrototypeOf(error) === other.TypeError.prototype,
    error instanceof other.TypeError,
    error instanceof TypeError
  ].join(":");
}
try {
  iteratorPrototype[Symbol.dispose].call({ return: 1 });
} catch (error) {
  disposeThrow = [
    Object.getPrototypeOf(error) === other.TypeError.prototype,
    error instanceof other.TypeError,
    error instanceof TypeError
  ].join(":");
}
[
  typeof C,
  C === Iterator,
  C.prototype === Iterator.prototype,
  Object.getPrototypeOf(C) === other.Function.prototype,
  Object.getPrototypeOf(iteratorPrototype) === other.Object.prototype,
  typeof iteratorPrototype[Symbol.iterator],
  iteratorPrototype[Symbol.iterator] === Iterator.prototype[Symbol.iterator],
  iteratorPrototype[Symbol.iterator]() === iteratorPrototype,
  thrown,
  typeof iteratorPrototype.toArray,
  iteratorPrototype.toArray === Iterator.prototype.toArray,
  toArrayThrow,
  typeof C.from,
  C.from === Iterator.from,
  iteratorPrototype.constructor === C,
  typeof constructorDesc.get,
  typeof constructorDesc.set,
  constructorDesc.get === mainConstructorDesc.get,
  constructorDesc.set === mainConstructorDesc.set,
  typeof iteratorPrototype[Symbol.dispose],
  iteratorPrototype[Symbol.dispose] === Iterator.prototype[Symbol.dispose],
  iteratorPrototype[Symbol.toStringTag],
  typeof tagDesc.get,
  typeof tagDesc.set,
  tagDesc.get === mainTagDesc.get,
  tagDesc.set === mainTagDesc.set,
  disposeThrow,
  helperResults.join(",")
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("cross-realm Iterator constructor call should throw in defining realm");
        assert!(
            outcome.note.contains(
                "string(function|false|false|true|true|function|false|true|true:true:false|function|false|true:true:false|function|false|true|function|function|false|false|function|false|Iterator|function|function|false|false|true:true:false|toArray:function:false,forEach:function:false,every:function:false,some:function:false,find:function:false,reduce:function:false,map:function:false,filter:function:false,flatMap:function:false,take:function:false,drop:function:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_eager_iterator_helpers_validate_arguments_in_other_realm() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let iteratorPrototype = other.Iterator.prototype;
let helperNames = ["toArray", "forEach", "every", "some", "find", "reduce"];
let callbackNames = ["forEach", "every", "some", "find", "reduce", "map", "filter", "flatMap"];
let nullishResults = [];
let callbackResults = [];
let rangeResults = [];
function collectNullishResults() {
for (let i = 0; i < 11; i++) {
  let name = i < 6 ? helperNames[i] : ["map", "filter", "flatMap", "take", "drop"][i - 6];
  try {
    iteratorPrototype[name].call(null, function() { return true; });
  } catch (error) {
    nullishResults.push(name + ":" + (Object.getPrototypeOf(error) === other.TypeError.prototype) + ":" + (error instanceof other.TypeError) + ":" + (error instanceof TypeError));
  }
}
}
function collectCallbackResults() {
for (let i = 0; i < callbackNames.length; i++) {
  let name = callbackNames[i];
  try {
    iteratorPrototype[name].call(iteratorPrototype, null);
  } catch (error) {
    callbackResults.push(name + ":" + (Object.getPrototypeOf(error) === other.TypeError.prototype) + ":" + (error instanceof other.TypeError) + ":" + (error instanceof TypeError));
  }
}
}
function collectRangeResults() {
for (let i = 0; i < 2; i++) {
  let name = i === 0 ? "take" : "drop";
  try {
    iteratorPrototype[name].call(iteratorPrototype);
  } catch (error) {
    rangeResults.push(name + ":" + (Object.getPrototypeOf(error) === other.RangeError.prototype) + ":" + (error instanceof other.RangeError) + ":" + (error instanceof RangeError));
  }
}
}
collectNullishResults();
collectCallbackResults();
collectRangeResults();
[
  nullishResults.join(","),
  callbackResults.join(","),
  rangeResults.join(",")
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("cross-realm eager Iterator helpers should use their defining realm");
        assert!(
            outcome.note.contains(
                "string(toArray:true:true:false,forEach:true:true:false,every:true:true:false,some:true:true:false,find:true:true:false,reduce:true:true:false,map:true:true:false,filter:true:true:false,flatMap:true:true:false,take:true:true:false,drop:true:true:false|forEach:true:true:false,every:true:true:false,some:true:true:false,find:true:true:false,reduce:true:true:false,map:true:true:false,filter:true:true:false,flatMap:true:true:false|take:true:true:false,drop:true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_eager_iterator_helpers_validate_results_in_other_realm() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let iteratorPrototype = other.Iterator.prototype;
let helperNames = ["toArray", "forEach", "every", "some", "find", "reduce"];
let nextMethodResults = [];
let nextResultResults = [];
function collectNextMethodResults() {
for (let i = 0; i < helperNames.length; i++) {
  let name = helperNames[i];
  try {
    if (name === "toArray") {
      iteratorPrototype[name].call({ next: null });
    } else {
      iteratorPrototype[name].call({ next: null }, function() { return true; });
    }
  } catch (error) {
    nextMethodResults.push(name + ":" + (Object.getPrototypeOf(error) === other.TypeError.prototype) + ":" + (error instanceof other.TypeError) + ":" + (error instanceof TypeError));
  }
}
}
function collectNextResultResults() {
for (let i = 0; i < helperNames.length; i++) {
  let name = helperNames[i];
  try {
    if (name === "toArray") {
      iteratorPrototype[name].call({ next: function() { return 1; } });
    } else {
      iteratorPrototype[name].call({ next: function() { return 1; } }, function() { return true; });
    }
  } catch (error) {
    nextResultResults.push(name + ":" + (Object.getPrototypeOf(error) === other.TypeError.prototype) + ":" + (error instanceof other.TypeError) + ":" + (error instanceof TypeError));
  }
}
}
collectNextMethodResults();
collectNextResultResults();
let reduceEmpty = "none";
try {
  iteratorPrototype.reduce.call({ next: function() { return { done: true }; } }, function(a, b) { return a; });
} catch (error) {
  reduceEmpty = [(Object.getPrototypeOf(error) === other.TypeError.prototype), error instanceof other.TypeError, error instanceof TypeError].join(":");
}
[nextMethodResults.join(","), nextResultResults.join(","), reduceEmpty].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("cross-realm eager Iterator results should use their defining realm");
        assert!(
            outcome.note.contains(
                "string(toArray:true:true:false,forEach:true:true:false,every:true:true:false,some:true:true:false,find:true:true:false,reduce:true:true:false|toArray:true:true:false,forEach:true:true:false,every:true:true:false,some:true:true:false,find:true:true:false,reduce:true:true:false|true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_lazy_iterator_helpers_use_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let iteratorPrototype = other.Iterator.prototype;
let nextResults = [];
let returnResults = [];
let checks = [
  ["map", iteratorPrototype.map.call({ next: function() { return 1; } }, function(value) { return value; })],
  ["filter", iteratorPrototype.filter.call({ next: function() { return 1; } }, function(value) { return true; })],
  ["flatMap", iteratorPrototype.flatMap.call({ next: function() { return 1; } }, function(value) { return [value]; })],
  ["take", iteratorPrototype.take.call({ next: function() { return 1; } }, 1)],
  ["drop", iteratorPrototype.drop.call({ next: function() { return 1; } }, 0)]
];
for (let i = 0; i < checks.length; i++) {
  try {
    checks[i][1].next();
  } catch (error) {
    nextResults.push(
      checks[i][0] + ":" +
      (Object.getPrototypeOf(error) === other.TypeError.prototype) + ":" +
      (error instanceof other.TypeError) + ":" +
      (error instanceof TypeError)
    );
  }
}
for (let i = 0; i < checks.length; i++) {
  try {
    checks[i][1].return.call({});
  } catch (error) {
    returnResults.push(
      checks[i][0] + ":" +
      (Object.getPrototypeOf(error) === other.TypeError.prototype) + ":" +
      (error instanceof other.TypeError) + ":" +
      (error instanceof TypeError)
    );
  }
}
[nextResults.join(","), returnResults.join(",")].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("cross-realm lazy Iterator helpers should use their defining realm");
        assert!(
            outcome.note.contains(
                "string(map:true:true:false,filter:true:true:false,flatMap:true:true:false,take:true:true:false,drop:true:true:false|map:true:true:false,filter:true:true:false,flatMap:true:true:false,take:true:true:false,drop:true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_iterator_from_uses_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let C = other.Iterator;
let throwResults = [];
function recordThrow(label, thunk) {
  try {
    thunk();
  } catch (error) {
    throwResults.push(
      label + ":" +
      (Object.getPrototypeOf(error) === other.TypeError.prototype) + ":" +
      (error instanceof other.TypeError) + ":" +
      (error instanceof TypeError)
    );
  }
}
recordThrow("null", function() { C.from(null); });
recordThrow("method", function() {
  let value = {};
  value[Symbol.iterator] = 1;
  C.from(value);
});
recordThrow("methodResult", function() {
  let value = {};
  value[Symbol.iterator] = function() { return 1; };
  C.from(value);
});
recordThrow("nextMethod", function() {
  let value = { next: 1 };
  value[Symbol.iterator] = function() { return this; };
  C.from(value).next();
});
recordThrow("nextResult", function() {
  let value = { next: function() { return 1; } };
  value[Symbol.iterator] = function() { return this; };
  C.from(value).next();
});
recordThrow("nextReceiver", function() {
  let value = { next: function() { return { done: true }; } };
  value[Symbol.iterator] = function() { return this; };
  C.from(value).next.call({});
});
recordThrow("returnMethod", function() {
  let value = {
    next: function() { return { done: false, value: 1 }; },
    return: 1
  };
  value[Symbol.iterator] = function() { return this; };
  C.from(value).return();
});
recordThrow("returnResult", function() {
  let value = {
    next: function() { return { done: false, value: 1 }; },
    return: function() { return 1; }
  };
  value[Symbol.iterator] = function() { return this; };
  C.from(value).return();
});
recordThrow("returnReceiver", function() {
  let value = {
    next: function() { return { done: false, value: 1 }; },
    return: function() { return { done: true }; }
  };
  value[Symbol.iterator] = function() { return this; };
  C.from(value).return.call({});
});
let nextThis = "unset";
let result = { done: false, value: 42 };
let value = {
  next: function() {
    nextThis = this === value;
    return result;
  }
};
value[Symbol.iterator] = function() { return this; };
let wrapper = C.from(value);
let nextResult = wrapper.next();
[
  throwResults.join(","),
  nextThis,
  nextResult === result,
  typeof wrapper.next
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("cross-realm Iterator.from should use its defining realm");
        assert!(
            outcome.note.contains(
                "string(null:true:true:false,method:true:true:false,methodResult:true:true:false,nextMethod:true:true:false,nextResult:true:true:false,nextReceiver:true:true:false,returnMethod:true:true:false,returnResult:true:true:false,returnReceiver:true:true:false|true|true|function)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_uses_realm_local_object_and_array_prototypes() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let array = new other.Array();
let object = new other.Object();
object.own = 1;
[
  other.Object === Object,
  other.Array === Array,
  other.Function.prototype === Function.prototype,
  Object.getPrototypeOf(other.Object) === other.Function.prototype,
  Object.getPrototypeOf(other.Array) === other.Function.prototype,
  other.Object.prototype === Object.prototype,
  other.Array.prototype === Array.prototype,
  Object.getPrototypeOf(other.Array.prototype) === other.Object.prototype,
  Object.getPrototypeOf(array) === other.Array.prototype,
  typeof other.Object.prototype.hasOwnProperty,
  typeof other.Object.prototype.propertyIsEnumerable,
  typeof other.Object.prototype.isPrototypeOf,
  typeof other.Object.prototype.toString,
  typeof other.Object.prototype.toLocaleString,
  typeof other.Object.prototype.valueOf,
  other.Object.prototype.hasOwnProperty === Object.prototype.hasOwnProperty,
  other.Object.prototype.propertyIsEnumerable === Object.prototype.propertyIsEnumerable,
  other.Object.prototype.isPrototypeOf === Object.prototype.isPrototypeOf,
  other.Object.prototype.toString === Object.prototype.toString,
  other.Object.prototype.toLocaleString === Object.prototype.toLocaleString,
  other.Object.prototype.valueOf === Object.prototype.valueOf,
  other.Object.prototype.hasOwnProperty.call(object, "own"),
  other.Object.prototype.propertyIsEnumerable.call(object, "own"),
  other.Object.prototype.isPrototypeOf.call(other.Object.prototype, object),
  other.Object.prototype.toString.call(object),
  other.Object.prototype.toLocaleString.call(object),
  other.Object.prototype.valueOf.call(object) === object
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm should use realm-local Object/Array prototypes");
        assert!(
            outcome
                .note
                .contains("string(false|false|false|true|true|false|false|true|true|function|function|function|function|function|function|false|false|false|false|false|false|true|true|true|[object Object]|[object Object]|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_uses_realm_local_function_constructor_and_prototype() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
function add(a, b) { return this.base + a + b; }
let receiver = { base: 10 };
let bound = other.Function.prototype.bind.call(add, receiver, 2);
[
  other.Function === Function,
  other.Function.prototype === Function.prototype,
  Object.getPrototypeOf(other.Function) === other.Function.prototype,
  Object.getPrototypeOf(other.Function.prototype) === other.Object.prototype,
  other.Function.prototype.constructor === other.Function,
  typeof other.Function.prototype.call,
  typeof other.Function.prototype.apply,
  typeof other.Function.prototype.bind,
  typeof other.Function.prototype.toString,
  other.Function.prototype.call === Function.prototype.call,
  other.Function.prototype.apply === Function.prototype.apply,
  other.Function.prototype.bind === Function.prototype.bind,
  other.Function.prototype.toString === Function.prototype.toString,
  other.Function.prototype.call.call(add, receiver, 1, 2),
  other.Function.prototype.apply.call(add, receiver, [3, 4]),
  bound(3)
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm should use realm-local Function constructor and prototype");
        assert!(
            outcome
                .note
                .contains("string(false|false|true|true|true|function|function|function|function|false|false|false|false|13|17|15)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_uses_realm_local_array_iterator_prototype() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let otherIter = other.Array.prototype.values.call(new other.Array(1));
let thisIter = Array.prototype.values.call([]);
let otherArrayIteratorPrototype = Object.getPrototypeOf(otherIter);
let thisArrayIteratorPrototype = Object.getPrototypeOf(thisIter);
let otherIteratorPrototype = Object.getPrototypeOf(otherArrayIteratorPrototype);
let thisIteratorPrototype = Object.getPrototypeOf(thisArrayIteratorPrototype);
[
  otherArrayIteratorPrototype === thisArrayIteratorPrototype,
  otherIteratorPrototype === thisIteratorPrototype,
  Object.getPrototypeOf(otherIteratorPrototype) === other.Object.prototype,
  typeof otherIter.next,
  otherArrayIteratorPrototype.next === thisArrayIteratorPrototype.next,
  otherArrayIteratorPrototype[Symbol.iterator] === thisArrayIteratorPrototype[Symbol.iterator],
  otherIter[Symbol.iterator]() === otherIter
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm should use a realm-local Array Iterator prototype graph");
        assert!(
            outcome
                .note
                .contains("string(false|false|true|function|false|false|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_array_species_create_uses_receiver_realm_constructor() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let array = new other.Array(2);
array[0] = 41;
array[1] = 7;
let mapped = other.Array.prototype.map.call(array, function(value) { return value + 1; });
let filtered = other.Array.prototype.filter.call(array, function(value) { return value > 10; });
[
  array.constructor === other.Array,
  other.Array.prototype.constructor === other.Array,
  other.Array[Symbol.species] === other.Array,
  Object.getPrototypeOf(mapped) === other.Array.prototype,
  Object.getPrototypeOf(mapped) === Array.prototype,
  mapped.constructor === other.Array,
  mapped.length,
  mapped[0],
  mapped[1],
  Object.getPrototypeOf(filtered) === other.Array.prototype,
  Object.getPrototypeOf(filtered) === Array.prototype,
  filtered.constructor === other.Array,
  filtered.length,
  filtered[0]
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("ArraySpeciesCreate should allocate with the receiver realm constructor");
        assert!(
            outcome
                .note
                .contains("string(true|true|true|true|false|true|2|42|8|true|false|true|1|41)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_exposes_array_mutator_and_locale_methods() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let array = new other.Array(2);
array[0] = 1;
array[1] = 2;
let pushedLength = other.Array.prototype.push.call(array, 3);
let popped = other.Array.prototype.pop.call(array);
[
  typeof other.Array.prototype.toLocaleString,
  typeof other.Array.prototype.push,
  typeof other.Array.prototype.pop,
  other.Array.prototype.toLocaleString === Array.prototype.toLocaleString,
  other.Array.prototype.push === Array.prototype.push,
  other.Array.prototype.pop === Array.prototype.pop,
  other.Array.prototype.toLocaleString.call(array),
  pushedLength,
  popped,
  array.length,
  array[0],
  array[1],
  Object.getPrototypeOf(array) === other.Array.prototype
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm should expose Array prototype mutator and locale methods");
        assert!(
            outcome.note.contains(
                "string(function|function|function|false|false|false|1,2|3|3|2|1|2|true)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_allocates_distinct_global_and_intrinsics_per_realm() {
        let outcome = engine()
            .run_script(
                r#"
let first = __porfCreateRealm();
let second = __porfCreateRealm();
[
  first === second,
  first.global === second.global,
  first.global.Object === second.global.Object,
  first.global.Object.prototype === second.global.Object.prototype,
  Object.getPrototypeOf(first.global.Object) === Object.getPrototypeOf(second.global.Object)
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("each synthetic realm should allocate a distinct global and intrinsic graph");
        assert!(
            outcome
                .note
                .contains("string(false|false|false|false|false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_uses_realm_local_typed_array_prototypes() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let otherTypedArrayPrototype = Object.getPrototypeOf(other.Uint8Array.prototype);
let thisTypedArrayPrototype = Object.getPrototypeOf(Uint8Array.prototype);
let view = new other.Uint8Array(2);
[
  other.Uint8Array === Uint8Array,
  other.Uint8Array.prototype === Uint8Array.prototype,
  otherTypedArrayPrototype === thisTypedArrayPrototype,
  Object.getPrototypeOf(otherTypedArrayPrototype) === other.Object.prototype,
  Object.getPrototypeOf(view) === other.Uint8Array.prototype,
  Object.getPrototypeOf(other.Uint8Array) === Object.getPrototypeOf(Uint8Array)
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm should use realm-local typed array prototypes");
        assert!(
            outcome
                .note
                .contains("string(false|false|false|true|true|false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_uses_realm_local_typed_array_constructor_family() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let names = [
  "Int8Array", "Uint8Array", "Uint8ClampedArray",
  "Int16Array", "Uint16Array", "Int32Array", "Uint32Array",
  "Float32Array", "Float64Array", "BigInt64Array", "BigUint64Array"
];
let otherTypedArrayPrototype = Object.getPrototypeOf(other.Uint8Array.prototype);
let ok = true;

for (let i = 0; i < names.length; i++) {
  let name = names[i];
  let C = other[name];
  let ThisC = globalThis[name];
  let isBigInt = name === "BigInt64Array" || name === "BigUint64Array";
  let view = new C(isBigInt ? [0n] : [0]);
  ok =
    ok &&
    C !== ThisC &&
    C.prototype !== ThisC.prototype &&
    Object.getPrototypeOf(C.prototype) === otherTypedArrayPrototype &&
    Object.getPrototypeOf(view) === C.prototype &&
    typeof view[0] === (isBigInt ? "bigint" : "number") &&
    view instanceof C &&
    !(view instanceof ThisC);
}

ok;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm should use realm-local typed array constructor family");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_uses_realm_local_array_buffer_and_data_view_prototypes() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let buffer = new other.ArrayBuffer(8);
let view = new other.DataView(buffer);
[
  other.ArrayBuffer === ArrayBuffer,
  other.ArrayBuffer.prototype === ArrayBuffer.prototype,
  Object.getPrototypeOf(other.ArrayBuffer) === other.Function.prototype,
  Object.getPrototypeOf(other.ArrayBuffer.prototype) === other.Object.prototype,
  other.ArrayBuffer.prototype.constructor === other.ArrayBuffer,
  Object.getPrototypeOf(buffer) === other.ArrayBuffer.prototype,
  buffer instanceof other.ArrayBuffer,
  buffer instanceof ArrayBuffer,
  other.DataView === DataView,
  other.DataView.prototype === DataView.prototype,
  Object.getPrototypeOf(other.DataView) === other.Function.prototype,
  Object.getPrototypeOf(other.DataView.prototype) === other.Object.prototype,
  other.DataView.prototype.constructor === other.DataView,
  Object.getPrototypeOf(view) === other.DataView.prototype,
  view instanceof other.DataView,
  view instanceof DataView,
  other.ArrayBuffer.isView === ArrayBuffer.isView,
  typeof other.ArrayBuffer.isView,
  other.ArrayBuffer.isView(view),
  other.ArrayBuffer.isView(new other.Uint8Array(buffer)),
  other.ArrayBuffer.isView(buffer),
  other.ArrayBuffer.isView({})
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm should use realm-local ArrayBuffer and DataView prototypes");
        assert!(
            outcome.note.contains(
                "string(false|false|true|true|true|true|true|false|false|false|true|true|true|true|true|false|false|function|true|true|false|false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_uses_realm_local_bigint_prototype() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
[
  other.BigInt.prototype === BigInt.prototype,
  Object.getPrototypeOf(other.BigInt.prototype) === other.Object.prototype,
  other.BigInt.prototype.constructor === other.BigInt
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm should use realm-local BigInt prototype");
        assert!(
            outcome.note.contains("string(false|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_exposes_bigint_static_methods() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
[
  other.BigInt.asIntN === BigInt.asIntN,
  typeof other.BigInt.asIntN,
  typeof other.BigInt.asUintN,
  other.BigInt.asIntN(4, 15n).toString(),
  other.BigInt.asUintN(4, -1n).toString()
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm BigInt constructor should expose static methods");
        assert!(
            outcome
                .note
                .contains("string(false|function|function|-1|15)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_exposes_number_static_methods() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
[
  other.Number.isInteger === Number.isInteger,
  typeof other.Number.isInteger,
  typeof other.Number.isSafeInteger,
  typeof other.Number.isFinite,
  typeof other.Number.isNaN,
  typeof other.Number.parseInt,
  typeof other.Number.parseFloat,
  other.Number.isInteger(1),
  other.Number.isInteger(1.5),
  other.Number.isSafeInteger(9007199254740991),
  other.Number.isSafeInteger(9007199254740992),
  other.Number.isFinite(3),
  other.Number.isFinite(Infinity),
  other.Number.isNaN(NaN),
  other.Number.isNaN("NaN"),
  other.Number.parseInt("ff", 16),
  other.Number.parseFloat("1.25x"),
  typeof other.parseInt,
  typeof other.parseFloat,
  other.parseInt("10", 2),
  other.parseFloat("1.5e1"),
  other.Number.MAX_SAFE_INTEGER,
  other.Number.MIN_SAFE_INTEGER,
  other.Number.POSITIVE_INFINITY === Infinity,
  other.Number.NEGATIVE_INFINITY === -Infinity,
  other.Number.EPSILON > 0 && other.Number.EPSILON < 1,
  other.Number.MIN_VALUE > 0,
  other.Number.MAX_VALUE > 1,
  other.Number.NaN === other.Number.NaN,
  Object.getOwnPropertyDescriptor(other.Number, "MAX_SAFE_INTEGER").writable,
  Object.getOwnPropertyDescriptor(other.Number, "MAX_SAFE_INTEGER").enumerable,
  Object.getOwnPropertyDescriptor(other.Number, "MAX_SAFE_INTEGER").configurable
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm Number constructor should expose static methods");
        assert!(
            outcome.note.contains(
                "string(false|function|function|function|function|function|function|true|false|true|false|true|false|true|false|255|1.25|function|function|2|15|9007199254740991|-9007199254740991|true|true|true|true|true|false|false|false|false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_uses_realm_local_date_constructor_and_prototype() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let date = new other.Date(0);
let thisDate = new Date(0);
let setResult = date.setUTCFullYear(2001, 1, 3);
[
  other.Date === Date,
  other.Date.prototype === Date.prototype,
  Object.getPrototypeOf(other.Date.prototype) === other.Object.prototype,
  Object.getPrototypeOf(date) === other.Date.prototype,
  typeof other.Date.now,
  typeof other.Date.UTC,
  typeof other.Date.prototype.setUTCFullYear,
  typeof other.Date.prototype.toUTCString,
  other.Date.prototype.toGMTString === other.Date.prototype.toUTCString,
  other.Date.now(),
  other.Date.UTC(1970, 0, 1),
  date.getTime(),
  date.getUTCFullYear(),
  setResult === date.getTime(),
  Date.prototype.getTime.call(date),
  other.Date.prototype.getTime.call(thisDate),
  other.Date.prototype.getUTCFullYear.call(thisDate)
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm should use realm-local Date constructor and prototype");
        assert!(
            outcome.note.contains(
                "string(false|false|true|true|function|function|function|function|true|0|0|981158400000|2001|true|981158400000|0|1970)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_uses_realm_local_error_constructors_and_prototypes() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let names = [
  "Error", "EvalError", "RangeError", "ReferenceError", "SyntaxError",
  "TypeError", "URIError", "AggregateError", "SuppressedError"
];
let results = [];

for (let i = 0; i < names.length; i++) {
  let name = names[i];
  let C = other[name];
  let error =
    name === "AggregateError" ? new C([], "m") :
    name === "SuppressedError" ? new C("e", "s", "m") :
    new C("m");
  let expectedParent =
    name === "Error" ? other.Object.prototype : other.Error.prototype;
  let expectedConstructorParent =
    name === "Error" ? other.Function.prototype : other.Error;

  results.push(
    name + "=" +
    (C === globalThis[name]) + ":" +
    (C.prototype === globalThis[name].prototype) + ":" +
    (Object.getPrototypeOf(C) === expectedConstructorParent) + ":" +
    (C.prototype.constructor === C) + ":" +
    (Object.getPrototypeOf(C.prototype) === expectedParent) + ":" +
    (Object.getPrototypeOf(error) === C.prototype) + ":" +
    (error instanceof C) + ":" +
    (error instanceof other.Error) + ":" +
    (error instanceof Error)
  );
}

results.join(",");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm should use realm-local error constructors and prototypes");
        assert!(
            outcome.note.contains(
                "string(Error=false:false:true:true:true:true:true:true:false,EvalError=false:false:true:true:true:true:true:true:false,RangeError=false:false:true:true:true:true:true:true:false,ReferenceError=false:false:true:true:true:true:true:true:false,SyntaxError=false:false:true:true:true:true:true:true:false,TypeError=false:false:true:true:true:true:true:true:false,URIError=false:false:true:true:true:true:true:true:false,AggregateError=false:false:true:true:true:true:true:true:false,SuppressedError=false:false:true:true:true:true:true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_uses_realm_local_error_prototype_to_string() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let otherError = new other.Error("m");
let thisError = new Error("m");
let object = new other.Object();
object.name = "OtherName";
object.message = "OtherMessage";
[
  other.Error.prototype.toString === Error.prototype.toString,
  typeof other.Error.prototype.toString,
  other.Error.prototype.toString.call(otherError),
  other.Error.prototype.toString.call(thisError),
  Error.prototype.toString.call(otherError),
  other.Error.prototype.toString.call(object)
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm should use realm-local Error.prototype.toString");
        assert!(
            outcome.note.contains(
                "string(false|function|Error: m|Error: m|Error: m|OtherName: OtherMessage)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_exposes_error_static_methods() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let error = new other.Error("m");
let typeError = new other.TypeError("m");
[
  other.Error.isError === Error.isError,
  typeof other.Error.isError,
  other.Error.isError(error),
  other.Error.isError(typeError),
  other.Error.isError(new Error("m")),
  other.Error.isError({}),
  other.Error.isError("m")
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm Error constructor should expose static methods");
        assert!(
            outcome
                .note
                .contains("string(false|function|true|true|true|false|false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_error_constructor_fallback_uses_new_target_realm() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let originalOtherErrorPrototype = other.Error.prototype;
other.Error.prototype = 7;
let error = Reflect.construct(Error, ["m"], other.Error);
let proxyError = Reflect.construct(Error, ["m"], new Proxy(other.Error, {}));
[
  Object.getPrototypeOf(error) === originalOtherErrorPrototype,
  Object.getPrototypeOf(error) === Error.prototype,
  Object.getPrototypeOf(error) === other.Error.prototype,
  Object.getPrototypeOf(proxyError) === originalOtherErrorPrototype,
  Object.getPrototypeOf(proxyError) === Error.prototype
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("cross-realm Error construction should fall back to newTarget realm intrinsic");
        assert!(
            outcome.note.contains("string(true|false|false|true|false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_revoked_proxy_throws_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let OProxy = other.global.Proxy;
let proxyObj = OProxy.revocable(function() {}, {});
let proxy = proxyObj.proxy;
proxyObj.revoke();
let sameRealm = (new TypeError("same")) instanceof TypeError;

try {
  proxy();
  "missing";
} catch (error) {
  sameRealm + ":" +
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("revoked cross-realm proxy call should throw in proxy function realm");
        assert!(
            outcome.note.contains("string(true:true:true:false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_borrowed_and_bound_builtin_throw_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let method = other.global.String.prototype.toString;
let bound = method.bind(null);
let direct = "missing";
let boundResult = "missing";

try {
  method.call(null);
} catch (error) {
  direct =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

try {
  bound();
} catch (error) {
  boundResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

direct + "|" + boundResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("borrowed and bound cross-realm builtins should throw in defining realm");
        assert!(
            outcome
                .note
                .contains("string(true:true:false|true:true:false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_array_flat_methods_use_receiver_realm_species() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let array = new other.Array(1);
let nested = new other.Array(1);
nested[0] = 5;
array[0] = nested;
let flat = other.Array.prototype.flat.call(array);
let flatMapped = other.Array.prototype.flatMap.call(array, function(value) { return value; });
[
  typeof other.Array.prototype.flat,
  Object.getPrototypeOf(flat) === other.Array.prototype,
  Object.getPrototypeOf(flat) === Array.prototype,
  flat.constructor === other.Array,
  flat.length,
  flat[0],
  typeof other.Array.prototype.flatMap,
  Object.getPrototypeOf(flatMapped) === other.Array.prototype,
  Object.getPrototypeOf(flatMapped) === Array.prototype,
  flatMapped.constructor === other.Array,
  flatMapped.length,
  flatMapped[0]
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("cross-realm flat/flatMap should use receiver realm species constructors");
        assert!(
            outcome
                .note
                .contains("string(function|true|false|true|1|5|function|true|false|true|1|5)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_array_flat_methods_throw_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let results = [];
let flat = other.global.Array.prototype.flat;
let flatMap = other.global.Array.prototype.flatMap;
let boundFlat = flat.bind(null);
let boundFlatMapNullish = flatMap.bind(null, function(value) { return value; });
let boundFlatMapBadMapper = flatMap.bind([], null);
let calls = [
  function() { return flat.call(null); },
  function() { return boundFlat(); },
  function() { return flatMap.call(null, function(value) { return value; }); },
  function() { return boundFlatMapNullish(); },
  function() { return flatMap.call([], null); },
  function() { return boundFlatMapBadMapper(); }
];
for (let i = 0; i < calls.length; i++) {
  try {
    calls[i]();
    results.push("missing");
  } catch (error) {
    results.push(
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError)
    );
  }
}
results.join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("cross-realm flat/flatMap throws should use the defining realm TypeError");
        assert!(
            outcome.note.contains(
                "string(true:true:false|true:true:false|true:true:false|true:true:false|true:true:false|true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_array_concat_uses_receiver_realm_species() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let array = new other.Array(1);
array[0] = 3;
let result = other.Array.prototype.concat.call(array, 4);
[
  typeof other.Array.prototype.concat,
  Object.getPrototypeOf(result) === other.Array.prototype,
  Object.getPrototypeOf(result) === Array.prototype,
  result.constructor === other.Array,
  result.length,
  result[0],
  result[1]
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("cross-realm concat should use receiver realm species constructors");
        assert!(
            outcome
                .note
                .contains("string(function|true|false|true|2|3|4)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_array_concat_throws_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let concat = other.global.Array.prototype.concat;
let boundNullish = concat.bind(null);
let calls = [
  function() { return concat.call(null); },
  function() { return boundNullish(); }
];
let results = [];
for (let i = 0; i < calls.length; i++) {
  try {
    calls[i]();
    results.push("missing");
  } catch (error) {
    results.push(
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError)
    );
  }
}
results.join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("cross-realm concat throws should use the defining realm TypeError");
        assert!(
            outcome
                .note
                .contains("string(true:true:false|true:true:false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_exposes_array_static_methods_with_receiver_realm_results() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let otherArray = new other.Array(1);
otherArray[0] = 9;
let fromResult = other.Array.from([1, 2]);
let ofResult = other.Array.of(3, 4);
[
  typeof other.Array.from,
  typeof other.Array.of,
  typeof other.Array.isArray,
  other.Array.isArray(otherArray),
  Array.isArray(otherArray),
  Object.getPrototypeOf(fromResult) === other.Array.prototype,
  Object.getPrototypeOf(fromResult) === Array.prototype,
  fromResult.constructor === other.Array,
  fromResult.length,
  fromResult[0],
  fromResult[1],
  Object.getPrototypeOf(ofResult) === other.Array.prototype,
  Object.getPrototypeOf(ofResult) === Array.prototype,
  ofResult.constructor === other.Array,
  ofResult.length,
  ofResult[0],
  ofResult[1]
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm Array statics should use the receiver realm constructor");
        assert!(
            outcome.note.contains(
                "string(function|function|function|true|true|true|false|true|2|1|2|true|false|true|2|3|4)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_exposes_object_static_methods() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let proto = new other.Object();
other.Object.defineProperty(proto, "inherited", { value: 1, enumerable: true });
let object = other.Object.create(proto);
other.Object.defineProperty(object, "own", { value: 2, enumerable: true });
let replacement = new other.Object();
let setResult = other.Object.setPrototypeOf(object, replacement);
let desc = other.Object.getOwnPropertyDescriptor(object, "own");
let keys = other.Object.keys(object);
[
  typeof other.Object.create,
  typeof other.Object.getPrototypeOf,
  typeof other.Object.setPrototypeOf,
  typeof other.Object.defineProperty,
  typeof other.Object.getOwnPropertyDescriptor,
  typeof other.Object.keys,
  other.Object.getPrototypeOf(proto) === other.Object.prototype,
  Object.getPrototypeOf(proto) === Object.prototype,
  setResult === object,
  other.Object.getPrototypeOf(object) === replacement,
  other.Object.getPrototypeOf(replacement) === other.Object.prototype,
  desc.value,
  desc.enumerable,
  desc.configurable,
  keys.length,
  keys[0]
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm Object statics should operate on the other realm graph");
        assert!(
            outcome.note.contains(
                "string(function|function|function|function|function|function|true|false|true|true|true|2|true|false|1|own)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_exposes_object_integrity_static_methods() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let object = new other.Object();
let defined = other.Object.defineProperties(object, {
  a: { value: 1, enumerable: true },
  hidden: { value: 2 }
});
let extensibleBefore = other.Object.isExtensible(object);
let prevented = other.Object.preventExtensions(object);
let extensibleAfter = other.Object.isExtensible(object);
let frozenTarget = new other.Object();
other.Object.defineProperty(frozenTarget, "value", { value: 3, writable: true, configurable: true });
let frozen = other.Object.freeze(frozenTarget);
[
  typeof other.Object.defineProperties,
  typeof other.Object.hasOwn,
  typeof other.Object.is,
  typeof other.Object.freeze,
  typeof other.Object.isFrozen,
  typeof other.Object.isSealed,
  typeof other.Object.isExtensible,
  typeof other.Object.preventExtensions,
  defined === object,
  other.Object.hasOwn(object, "a"),
  other.Object.hasOwn(object, "hidden"),
  other.Object.hasOwn(object, "missing"),
  other.Object.is(NaN, NaN),
  other.Object.is(0, -0),
  extensibleBefore,
  prevented === object,
  extensibleAfter,
  frozen === frozenTarget,
  other.Object.isFrozen(frozenTarget),
  other.Object.isSealed(frozenTarget),
  other.Object.isExtensible(frozenTarget)
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm Object integrity statics should operate on objects");
        assert!(
            outcome.note.contains(
                "string(function|function|function|function|function|function|function|function|true|true|true|false|true|false|true|true|false|true|true|true|false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_exposes_object_property_list_static_methods() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let symbol = Symbol("s");
let object = new other.Object();
other.Object.defineProperty(object, "a", { value: 1, enumerable: true });
other.Object.defineProperty(object, "hidden", { value: 2 });
other.Object.defineProperty(object, symbol, { value: 3, enumerable: true });
let names = other.Object.getOwnPropertyNames(object);
let symbols = other.Object.getOwnPropertySymbols(object);
let values = other.Object.values(object);
[
  typeof other.Object.getOwnPropertyNames,
  typeof other.Object.getOwnPropertySymbols,
  typeof other.Object.values,
  names.length,
  names[0],
  names[1],
  symbols.length,
  symbols[0] === symbol,
  values.length,
  values[0],
  values[1]
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm Object property-list statics should operate on objects");
        assert!(
            outcome
                .note
                .contains("string(function|function|function|2|a|hidden|1|true|2|1|3)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_exposes_reflect_object_methods() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let object = new other.Object();
let proto = new other.Object();
let constructed = other.Reflect.construct(other.Object, []);
let defineResult = other.Reflect.defineProperty(object, "a", { value: 1, enumerable: true, configurable: true });
let setResult = other.Reflect.set(object, "b", 2);
let getResult = other.Reflect.get(object, "a");
let hasResult = other.Reflect.has(object, "b");
let desc = other.Reflect.getOwnPropertyDescriptor(object, "a");
let setProtoResult = other.Reflect.setPrototypeOf(object, proto);
let reflectedProto = other.Reflect.getPrototypeOf(object);
let keys = other.Reflect.ownKeys(object);
let deleteResult = other.Reflect.deleteProperty(object, "a");
let beforePreventExtensible = other.Reflect.isExtensible(object);
let preventResult = other.Reflect.preventExtensions(object);
[
  typeof other.Reflect,
  Object.getPrototypeOf(other.Reflect) === other.Object.prototype,
  typeof other.Reflect.construct,
  typeof other.Reflect.apply,
  typeof other.Reflect.get,
  typeof other.Reflect.getPrototypeOf,
  typeof other.Reflect.getOwnPropertyDescriptor,
  typeof other.Reflect.set,
  typeof other.Reflect.has,
  typeof other.Reflect.defineProperty,
  typeof other.Reflect.deleteProperty,
  typeof other.Reflect.isExtensible,
  typeof other.Reflect.preventExtensions,
  typeof other.Reflect.setPrototypeOf,
  typeof other.Reflect.ownKeys,
  Object.getPrototypeOf(constructed) === other.Object.prototype,
  defineResult,
  setResult,
  getResult,
  hasResult,
  desc.value,
  desc.enumerable,
  setProtoResult,
  reflectedProto === proto,
  Object.getPrototypeOf(object) === proto,
  keys.length,
  keys[0],
  keys[1],
  deleteResult,
  other.Reflect.has(object, "a"),
  beforePreventExtensible,
  preventResult,
  Object.isExtensible(object),
  other.Reflect.isExtensible(object)
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm Reflect object should expose object meta operations");
        assert!(
            outcome.note.contains(
                "string(object|true|function|function|function|function|function|function|function|function|function|function|function|function|function|true|true|true|1|true|1|true|true|true|true|2|a|b|true|false|true|true|false|false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_exposes_global_function_properties() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let infinityDesc = Object.getOwnPropertyDescriptor(other, "Infinity");
let nanDesc = Object.getOwnPropertyDescriptor(other, "NaN");
let undefinedDesc = Object.getOwnPropertyDescriptor(other, "undefined");
let globalThisDesc = Object.getOwnPropertyDescriptor(other, "globalThis");
let evalThrow = "missing";
try {
  other.eval("1 + 1");
} catch (error) {
  evalThrow =
    (Object.getPrototypeOf(error) === other.TypeError.prototype) + ":" +
    (error instanceof other.TypeError) + ":" +
    (error instanceof TypeError);
}
[
  other.Infinity === Infinity,
  other.NaN === other.NaN,
  other.undefined === undefined,
  other.globalThis === other,
  other.globalThis === globalThis,
  infinityDesc.writable,
  infinityDesc.enumerable,
  infinityDesc.configurable,
  nanDesc.writable,
  nanDesc.enumerable,
  nanDesc.configurable,
  undefinedDesc.writable,
  undefinedDesc.enumerable,
  undefinedDesc.configurable,
  globalThisDesc.writable,
  globalThisDesc.enumerable,
  globalThisDesc.configurable,
  other.eval === eval,
  typeof other.eval,
  other.eval(7),
  evalThrow,
  other.isFinite === isFinite,
  other.isNaN === isNaN,
  other.escape === escape,
  other.unescape === unescape,
  typeof other.isFinite,
  typeof other.isNaN,
  typeof other.escape,
  typeof other.unescape,
  other.isFinite("3"),
  other.isFinite("x"),
  other.isNaN("x"),
  other.isNaN("3"),
  other.unescape(other.escape("a b")) === "a b"
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm global object should expose global function properties");
        assert!(
            outcome.note.contains(
                "string(true|false|true|true|false|false|false|false|false|false|false|false|false|false|true|false|true|false|function|7|true:true:false|false|false|false|false|function|function|function|function|true|false|true|false|true)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_script_var_and_function_properties_are_enumerable() {
        let outcome = engine()
            .run_script(
                r#"
var variable;
function declared() {}
let variableDescriptor = Object.getOwnPropertyDescriptor(globalThis, "variable");
let functionDescriptor = Object.getOwnPropertyDescriptor(globalThis, "declared");
[
  variableDescriptor.writable,
  variableDescriptor.enumerable,
  variableDescriptor.configurable,
  functionDescriptor.writable,
  functionDescriptor.enumerable,
  functionDescriptor.configurable
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("script global declarations should create ordinary global properties");
        assert!(
            outcome
                .note
                .contains("string(true|true|false|true|true|false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_arguments_inherits_object_to_string() {
        let outcome = engine()
            .run_script(
                r#"
function describeArguments() {
  return arguments.toString();
}
describeArguments(1, 2);
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("arguments.toString should execute");
        assert!(
            outcome.note.contains("string([object Arguments])"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_exposes_math_object_methods_and_constants() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
[
  other.Math === Math,
  Object.getPrototypeOf(other.Math) === other.Object.prototype,
  Object.prototype.toString.call(other.Math),
  typeof other.Math.max,
  typeof other.Math.pow,
  other.Math.max(1, 7, 3),
  other.Math.pow(2, 5),
  other.Math.trunc(3.9),
  other.Math.PI > 3 && other.Math.PI < 4
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm Math object should expose Math operations");
        assert!(
            outcome
                .note
                .contains("string(false|true|[object Math]|function|function|7|32|3|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_exposes_json_object_methods() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let raw = other.JSON.rawJSON("1");
[
  other.JSON === JSON,
  Object.getPrototypeOf(other.JSON) === other.Object.prototype,
  Object.prototype.toString.call(other.JSON),
  typeof other.JSON.parse,
  typeof other.JSON.stringify,
  typeof other.JSON.rawJSON,
  typeof other.JSON.isRawJSON,
  other.JSON.parse("2"),
  other.JSON.stringify({ a: 1 }),
  other.JSON.isRawJSON(raw),
  other.JSON.isRawJSON({})
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm JSON object should expose JSON operations");
        assert!(
            outcome.note.contains(
                "string(false|true|[object JSON]|function|function|function|function|2|{\"a\":1}|true|false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_stored_builtin_retains_defining_realm() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let method = other.global.String.prototype.toString;
let holder = { method: method };
let array = [method];
let results = [];

for (let i = 0; i < 2; i++) {
  let stored = i === 0 ? holder.method : array[0];
  try {
    stored.call(null);
    results.push("missing");
  } catch (error) {
    results.push(
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError)
    );
  }
}

results.join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("stored cross-realm builtins should retain their defining realm");
        assert!(
            outcome
                .note
                .contains("string(true:true:false|true:true:false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_proxy_wrapped_builtin_throws_defining_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let method = other.global.String.prototype.toString;
let localProxy = new Proxy(method, {});
let otherProxy = new other.global.Proxy(method, {});
let localResult = "missing";
let otherResult = "missing";

try {
  localProxy.call(null);
} catch (error) {
  localResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

try {
  otherProxy.call(null);
} catch (error) {
  otherResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

localResult + "|" + otherResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("proxy-wrapped cross-realm builtin should throw in target defining realm");
        assert!(
            outcome
                .note
                .contains("string(true:true:false|true:true:false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_regexp_escape_throws_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let escape = other.global.RegExp.escape;
let bound = escape.bind(null);
let direct = "missing";
let boundResult = "missing";

try {
  escape(1);
} catch (error) {
  direct =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

try {
  bound(1);
} catch (error) {
  boundResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

direct + "|" + boundResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("borrowed and bound cross-realm RegExp.escape should throw in defining realm");
        assert!(
            outcome
                .note
                .contains("string(true:true:false|true:true:false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_uses_realm_local_regexp_constructor_and_prototype() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
[
  other.RegExp === RegExp,
  other.RegExp.prototype === RegExp.prototype,
  Object.getPrototypeOf(other.RegExp) === other.Function.prototype,
  Object.getPrototypeOf(other.RegExp.prototype) === other.Object.prototype,
  other.RegExp.prototype.constructor === other.RegExp,
  typeof other.RegExp.escape,
  other.RegExp.escape === RegExp.escape
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm should use realm-local RegExp constructor and prototype");
        assert!(
            outcome
                .note
                .contains("string(false|false|true|true|true|function|false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_regexp_source_getter_rejects_cross_realm_prototypes() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let get = Object.getOwnPropertyDescriptor(RegExp.prototype, "source").get;
let otherGet = Object.getOwnPropertyDescriptor(other.RegExp.prototype, "source").get;
let primaryError = false;
let otherError = false;

try {
  get.call(other.RegExp.prototype);
} catch (error) {
  primaryError = Object.getPrototypeOf(error) === TypeError.prototype;
}
try {
  otherGet.call(RegExp.prototype);
} catch (error) {
  otherError = Object.getPrototypeOf(error) === other.TypeError.prototype;
}
primaryError + "|" + otherError;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("RegExp source getters should reject another realm's prototype");
        assert!(
            outcome.note.contains("string(true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_regexp_match_all_iterator_uses_defining_realm() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let regexp = new other.global.RegExp("a", "g");
let iterator = other.global.String.prototype.matchAll.call("a", regexp);
let arrayIteratorPrototype =
  Object.getPrototypeOf(other.global.Array.prototype.values.call(new other.global.Array()));
let iteratorPrototype = Object.getPrototypeOf(iterator);
let next = iteratorPrototype.next;
let direct = "missing";

try {
  next.call({});
} catch (error) {
  direct =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

let nextResult = next.call(iterator);
[
  iteratorPrototype === arrayIteratorPrototype,
  iteratorPrototype === Object.getPrototypeOf(Array.prototype.values.call([])),
  direct,
  Object.getPrototypeOf(nextResult) === other.global.Object.prototype,
  Object.getPrototypeOf(nextResult) === Object.prototype
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("cross-realm RegExp matchAll iterator should use defining realm intrinsics");
        assert!(
            outcome
                .note
                .contains("string(true|false|true:true:false|true|false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_regexp_dot_all_matches_line_terminators() {
        let outcome = engine()
            .run_script(
                r#"[
  /^.$/s.test("\n"),
  /^.$/.test("\n"),
  /^.$/s.test("\u{10300}"),
  /^.$/su.test("\u{10300}")
].join("|");"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("dotAll matching should retain Unicode code-point behavior");
        assert!(
            outcome.note.contains("string(true|false|false|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_regexp_search_uses_compiled_named_groups() {
        let outcome = engine()
            .run_script(
                r#"[
  "xab".search(/(?<x>a)|(?<x>b)/),
  "xba".search(/(?<x>a)|(?<x>b)/)
].join("|");"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("RegExp search should execute compiled named-group programs");
        assert!(
            outcome.note.contains("string(1|1)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_regexp_open_capture_backreferences_match_empty() {
        let outcome = engine()
            .run_script(
                r#"
let named = /(?<a>\k<a>\w)../.exec("bab");
let numbered = /(\1\w)../.exec("bab");
[named[0], named.groups.a, numbered[0], numbered[1]].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("backreferences to open captures should match the empty string");
        assert!(
            outcome.note.contains("string(bab|b|bab|b)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_regexp_lookbehind_matches_in_reverse() {
        let outcome = engine()
            .run_script(
                r#"
let fixed = "abcdef".match(/(?<=\w{3})f/);
let greedy = "abcdef".match(/(?<=\w+)f/);
let negative = "abc123".match(/(?<!\d{3})c/);
let alternative = "xy".match(/(?<=(?<first>.)|(?<second>.))y/);
[fixed[0], greedy[0], negative[0], alternative.groups.first, alternative.groups.second].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("lookbehind should match its body in reverse without consuming input");
        assert!(
            outcome.note.contains("string(f|f|c|x|)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_regexp_identity_k_with_lookbehind_syntax() {
        for source in [
            r#"/\k<a>(?<=>)a/.test("k<a>a");"#,
            r#"/(?<=>)\k<a>/.test(">k<a>");"#,
            r#"/\k<a>(?<!a)a/.test("k<a>a");"#,
            r#"/(?<!a>)\k<a>/.test("k<a>");"#,
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|error| panic!("{source} should execute: {error:?}"));
            assert!(
                outcome.note.contains("boolean(true)"),
                "source: {source}; note: {}",
                outcome.note
            );
        }
    }

    #[test]
    fn wasm_backend_regexp_split_observes_symbol_match_getter_recompile() {
        let outcome = engine()
            .run_script(
                r#"
var regExp = /a/;
Object.defineProperty(regExp, Symbol.match, {
  get: function() {
    regExp.compile("b");
  }
});
regExp[Symbol.split]("abba").join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("RegExp split should observe Symbol.match getter side effects");
        assert!(
            outcome.note.contains("string(a||a)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_catches_regexp_test_abrupt_completions() {
        let outcome = engine()
            .run_script(
                r#"
var coercionThrow;
try {
  /a/.test({ toString: function() { throw "coercion"; } });
} catch (error) {
  coercionThrow = error;
}

var receiverThrow;
var ordinary = { test: RegExp.prototype.test };
try {
  ordinary.test("a");
} catch (error) {
  receiverThrow = error instanceof TypeError;
}

[coercionThrow, receiverThrow].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("RegExp test abrupt completions should reach the active catch");
        assert!(
            outcome.note.contains("string(coercion|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_regexp_subclass_exec_override_is_observable() {
        let outcome = engine()
            .run_script(
                r#"
class FakeRegExp extends RegExp {
  exec() {
    let result = ["ab", "a"];
    result.index = 0;
    result.groups = { a: "b" };
    return result;
  }
}

let regexp = new FakeRegExp();
let result = regexp.exec("ab");
[result.groups.a, "ab".replace(regexp, "$<a>")].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("RegExp subclass exec override should remain observable");
        assert!(
            outcome.note.contains("string(b|b)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_regexp_exec_resets_global_last_index_after_failure() {
        let source = r#"
let regexp = /a/g;
let reads = 0;
regexp.lastIndex = {
  valueOf() {
    reads += 1;
    return 42;
  }
};
let beyondEnd;
let beyondEndError = "none";
try {
  beyondEnd = regexp.exec("abc");
} catch (error) {
  beyondEndError = error.name;
}
let beyondEndState = [beyondEndError, beyondEnd === null, regexp.lastIndex, reads].join(":");

regexp.lastIndex = {
  valueOf() {
    reads += 1;
    return -1;
  }
};
let noMatch;
let noMatchError = "none";
try {
  noMatch = regexp.exec("nbc");
} catch (error) {
  noMatchError = error.name;
}
let noMatchState = [noMatchError, noMatch === null, regexp.lastIndex, reads].join(":");

let results = [];
for (let candidate of [Infinity, 2 ** 32, 5]) {
  for (let expression of [/./g, /./y, /./gy]) {
    expression.lastIndex = candidate;
    try {
      results.push((expression.exec("test") === null) + ":" + expression.lastIndex);
    } catch (error) {
      results.push(error.name + ":" + expression.lastIndex);
    }
  }
}
[beyondEndState, noMatchState, results.join(",")].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("failed global and sticky RegExp exec calls should reset lastIndex");
        assert!(
            outcome.note.contains(
                "string(none:true:0:1|none:true:0:2|true:0,true:0,true:0,true:0,true:0,true:0,true:0,true:0,true:0)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_regexp_exec_observes_last_index_once_and_writes_conditionally() {
        let source = r#"
function counter(reads) {
  return {
    valueOf() {
      reads.count += 1;
      return 0;
    }
  };
}

let failureReads = { count: 0 };
let failureCounter = counter(failureReads);
let nonglobalFailure = /a/;
nonglobalFailure.lastIndex = failureCounter;
let failed = nonglobalFailure.exec("nbc");

let successReads = { count: 0 };
let successCounter = counter(successReads);
let nonglobalSuccess = /./;
nonglobalSuccess.lastIndex = successCounter;
let succeeded = nonglobalSuccess.exec("abc");

let globalReads = { count: 0 };
let global = /./g;
global.lastIndex = counter(globalReads);
let globalResult = global.exec("abc");

let unicode = /./ug;
unicode.exec("𝌆");

[
  failed === null,
  nonglobalFailure.lastIndex === failureCounter,
  failureReads.count,
  succeeded[0],
  nonglobalSuccess.lastIndex === successCounter,
  successReads.count,
  globalResult[0],
  global.lastIndex,
  globalReads.count,
  unicode.lastIndex
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("RegExp exec should observe lastIndex with spec ordering");
        assert!(
            outcome
                .note
                .contains("string(true|true|1|a|true|1|a|1|1|2)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_let_binding_tracks_tag_after_cross_kind_assignment() {
        let outcome = engine()
            .run_script(
                r#"
let value = null;
value = { nested: { answer: 42 } };
value.nested.answer;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("mutable lexical bindings should retain the assigned runtime tag");
        assert!(
            outcome.note.contains("number(42)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_outlines_optional_computed_property_reads_after_nullish_check() {
        let outcome = engine()
            .run_script(
                r#"
let keyCalls = 0;
let key = { [Symbol.toPrimitive]() { keyCalls += 1; return "value"; } };
let object = { value: 42, undefined: 1, null: 2, true: 3, NaN: 4 };
let absent = null;
let skipped = absent?.[key];
let results = [
  object?.[key], object?.[undefined], object?.[null], object?.[true], object?.[NaN],
  object?.["value"], object?.[key], object?.[undefined], object?.[null],
  object?.[true], object?.[NaN], object?.["value"], object?.[key]
];
[skipped, keyCalls, results.join("|")].join(";");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect(
                "optional computed property reads should compile through the shared dispatcher",
            );
        assert!(
            outcome
                .note
                .contains("string(;3;42|1|2|3|4|42|42|1|2|3|4|42|42)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_adds_string_or_number_bindings_by_runtime_tag() {
        let outcome = engine()
            .run_script(
                r#"
function add(flag) {
  var value;
  if (flag) {
    value = 1;
  } else {
    value = "a";
  }
  return value + 1;
}
[add(true), add(false)].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("string-or-number bindings should use tagged addition");
        assert!(
            outcome.note.contains("string(2|a1)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_caches_and_freezes_tagged_template_objects() {
        let outcome = engine()
            .run_script(
                r#"
"use strict";
let first;
let second;
function tag(template) {
  if (first === undefined) first = template;
  else second = template;
}
function useTemplate() {
  tag`cooked`;
}
useTemplate();
useTemplate();

let assignmentResult = "missing";
try {
  first.extra = true;
} catch (error) {
  assignmentResult = error.name;
}
let rawDescriptor = Object.getOwnPropertyDescriptor(first, "raw");
let indexDescriptor = Object.getOwnPropertyDescriptor(first, "0");
[
  first === second,
  assignmentResult,
  Object.isFrozen(first),
  Object.isFrozen(first.raw),
  rawDescriptor.enumerable,
  rawDescriptor.writable,
  rawDescriptor.configurable,
  indexDescriptor.enumerable,
  indexDescriptor.writable,
  indexDescriptor.configurable
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("tagged template objects should be cached and deeply frozen");
        assert!(
            outcome
                .note
                .contains("string(true|TypeError|true|true|false|false|false|true|false|false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_tail_calls_tagged_templates_without_growing_the_wasm_stack() {
        let outcome = engine()
            .run_script(
                r#"
"use strict";
function recurse(_, remaining) {
  if (remaining === 0) return "finished";
  return recurse`${remaining - 1}`;
}
recurse(null, 20000);
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("tail-position tagged calls should not exhaust the Wasm stack");
        assert!(
            outcome.note.contains("string(finished)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_tail_calls_nested_return_expressions_without_growing_the_wasm_stack() {
        let outcome = engine()
            .run_script(
                r#"
"use strict";
function conditionalComma(remaining) {
  return remaining === 0 ? "conditional" : (0, conditionalComma(remaining - 1));
}
function logicalAnd(remaining) {
  return remaining !== 0 && logicalAnd(remaining - 1);
}
function logicalOr(remaining) {
  return remaining === 0 || logicalOr(remaining - 1);
}
function coalesce(remaining) {
  return (remaining === 0 ? "coalesce" : null) ?? coalesce(remaining - 1);
}
[
  conditionalComma(20000),
  logicalAnd(20000),
  logicalOr(20000),
  coalesce(20000)
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("nested return-position calls should not exhaust the Wasm stack");
        assert!(
            outcome
                .note
                .contains("string(conditional|false|true|coalesce)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_runtime_regexp_uses_lexical_source_with_finite_flags() {
        let outcome = engine()
            .run_script(
                r#"
let source = "(?<fst>.)(?<snd>.)|(?<thd>x)";
let result = "";
for (let flags of ["g", "gu"]) {
  let regexp = new RegExp(source, flags);
  result += "abcd".replace(regexp, "$<snd>$<fst>");
}
result;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("runtime RegExp should select lexical source and finite flags");
        assert!(
            outcome.note.contains("string(badcbadc)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_regexp_match_all_uses_compiled_named_groups() {
        let outcome = engine()
            .run_script(
                r#"
let iterator = "ba".matchAll(/(?<x>a)|(?<x>b)/g);
let first = iterator.next().value;
let second = iterator.next().value;
[
  first[0], first[1], first[2], first.groups.x,
  second[0], second[1], second[2], second.groups.x
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("RegExp matchAll should iterate compiled named-group programs");
        assert!(
            outcome.note.contains("string(b||b|b|a|a||a)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_date_get_time_throws_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let method = other.global.Date.prototype.getTime;
let bound = method.bind({});
let direct = "missing";
let boundResult = "missing";

try {
  method.call({});
} catch (error) {
  direct =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

try {
  bound();
} catch (error) {
  boundResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

direct + "|" + boundResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("borrowed and bound cross-realm Date.prototype.getTime should throw in defining realm");
        assert!(
            outcome
                .note
                .contains("string(true:true:false|true:true:false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_bigint_methods_throw_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let names = ["toString", "toLocaleString", "valueOf"];
let results = [];

for (let i = 0; i < names.length; i++) {
  let method = other.global.BigInt.prototype[names[i]];
  let bound = method.bind(null);
  let direct = "missing";
  let boundResult = "missing";

  try {
    method.call(null);
  } catch (error) {
    direct =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    bound();
  } catch (error) {
    boundResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  results.push(names[i] + "=" + direct + "|" + boundResult);
}

results.join(",");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("borrowed and bound cross-realm BigInt methods should throw in defining realm");
        assert!(
            outcome.note.contains(
                "string(toString=true:true:false|true:true:false,toLocaleString=true:true:false|true:true:false,valueOf=true:true:false|true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_number_methods_throw_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let names = ["toString", "toLocaleString", "valueOf"];
let results = [];

for (let i = 0; i < names.length; i++) {
  let method = other.global.Number.prototype[names[i]];
  let bound = method.bind(null);
  let direct = "missing";
  let boundResult = "missing";

  try {
    method.call(null);
  } catch (error) {
    direct =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    bound();
  } catch (error) {
    boundResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  results.push(names[i] + "=" + direct + "|" + boundResult);
}

results.join(",");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("borrowed and bound cross-realm Number methods should throw in defining realm");
        assert!(
            outcome.note.contains(
                "string(toString=true:true:false|true:true:false,toLocaleString=true:true:false|true:true:false,valueOf=true:true:false|true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_boolean_methods_throw_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let names = ["toString", "valueOf"];
let results = [];

for (let i = 0; i < names.length; i++) {
  let method = other.global.Boolean.prototype[names[i]];
  let bound = method.bind(null);
  let direct = "missing";
  let boundResult = "missing";

  try {
    method.call(null);
  } catch (error) {
    direct =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    bound();
  } catch (error) {
    boundResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  results.push(names[i] + "=" + direct + "|" + boundResult);
}

results.join(",");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect(
                "borrowed and bound cross-realm Boolean methods should throw in defining realm",
            );
        assert!(
            outcome.note.contains(
                "string(toString=true:true:false|true:true:false,valueOf=true:true:false|true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_uses_realm_local_boolean_constructor_and_prototype() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let value = new other.Boolean(true);
let thisValue = new Boolean(true);
[
  other.Boolean === Boolean,
  other.Boolean.prototype === Boolean.prototype,
  Object.getPrototypeOf(other.Boolean) === other.Function.prototype,
  Object.getPrototypeOf(other.Boolean.prototype) === other.Object.prototype,
  other.Boolean.prototype.constructor === other.Boolean,
  Object.getPrototypeOf(value) === other.Boolean.prototype,
  value instanceof other.Boolean,
  value instanceof Boolean,
  typeof other.Boolean.prototype.toString,
  typeof other.Boolean.prototype.valueOf,
  other.Boolean.prototype.toString.call(thisValue),
  other.Boolean.prototype.valueOf.call(thisValue)
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm should use realm-local Boolean constructor and prototype");
        assert!(
            outcome.note.contains(
                "string(false|false|true|true|true|true|true|false|function|function|true|true)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_function_to_string_throws_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let method = other.global.Function.prototype.toString;
let bound = method.bind({});
let direct = "missing";
let boundResult = "missing";

try {
  method.call({});
} catch (error) {
  direct =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

try {
  bound();
} catch (error) {
  boundResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

direct + "|" + boundResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("borrowed and bound cross-realm Function.prototype.toString should throw in defining realm");
        assert!(
            outcome
                .note
                .contains("string(true:true:false|true:true:false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_error_to_string_throws_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let method = other.global.Error.prototype.toString;
let bound = method.bind(null);
let direct = "missing";
let boundResult = "missing";

try {
  method.call(null);
} catch (error) {
  direct =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

try {
  bound();
} catch (error) {
  boundResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

direct + "|" + boundResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("borrowed and bound cross-realm Error.prototype.toString should throw in defining realm");
        assert!(
            outcome
                .note
                .contains("string(true:true:false|true:true:false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_object_value_of_throws_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let method = other.global.Object.prototype.valueOf;
let bound = method.bind(null);
let direct = "missing";
let boundResult = "missing";

try {
  method.call(null);
} catch (error) {
  direct =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

try {
  bound();
} catch (error) {
  boundResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

direct + "|" + boundResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("borrowed and bound cross-realm Object.prototype.valueOf should throw in defining realm");
        assert!(
            outcome
                .note
                .contains("string(true:true:false|true:true:false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_array_at_throws_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let method = other.global.Array.prototype.at;
let bound = method.bind(null, 0);
let direct = "missing";
let boundResult = "missing";

try {
  method.call(null, 0);
} catch (error) {
  direct =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

try {
  bound();
} catch (error) {
  boundResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

direct + "|" + boundResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect(
                "borrowed and bound cross-realm Array.prototype.at should throw in defining realm",
            );
        assert!(
            outcome
                .note
                .contains("string(true:true:false|true:true:false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_array_includes_throws_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let method = other.global.Array.prototype.includes;
let bound = method.bind(null, 1);
let direct = "missing";
let boundResult = "missing";

try {
  method.call(null, 1);
} catch (error) {
  direct =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

try {
  bound();
} catch (error) {
  boundResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

direct + "|" + boundResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect(
                "borrowed and bound cross-realm Array.prototype.includes should throw in defining realm",
            );
        assert!(
            outcome
                .note
                .contains("string(true:true:false|true:true:false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_array_mutator_and_locale_methods_throw_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let names = ["toLocaleString", "pop", "push"];
let results = [];

for (let i = 0; i < names.length; i++) {
  let method = other.global.Array.prototype[names[i]];
  let bound = method.bind(null, 1);
  let direct = "missing";
  let boundResult = "missing";

  try {
    method.call(null, 1);
  } catch (error) {
    direct =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    bound();
  } catch (error) {
    boundResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  results.push(names[i] + "=" + direct + "|" + boundResult);
}

results.join(",");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect(
                "borrowed and bound cross-realm Array mutator/locale methods should throw in defining realm",
            );
        assert!(
            outcome.note.contains(
                "string(toLocaleString=true:true:false|true:true:false,pop=true:true:false|true:true:false,push=true:true:false|true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_array_helper_methods_throw_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let names = ["indexOf", "lastIndexOf", "forEach"];
let results = [];

for (let i = 0; i < names.length; i++) {
  let method = other.global.Array.prototype[names[i]];
  let bound = method.bind(null, 1);
  let direct = "missing";
  let boundResult = "missing";

  try {
    method.call(null, 1);
  } catch (error) {
    direct =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    bound();
  } catch (error) {
    boundResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  results.push(names[i] + "=" + direct + "|" + boundResult);
}

results.join(",");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("borrowed and bound cross-realm Array helper methods should throw in defining realm");
        assert!(
            outcome.note.contains(
                "string(indexOf=true:true:false|true:true:false,lastIndexOf=true:true:false|true:true:false,forEach=true:true:false|true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_array_find_like_methods_throw_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let names = ["find", "findIndex", "findLast", "findLastIndex"];
let results = [];

for (let i = 0; i < names.length; i++) {
  let method = other.global.Array.prototype[names[i]];
  let boundNullish = method.bind(null, function() { return true; });
  let boundBadPredicate = method.bind([], null);
  let directNullish = "missing";
  let boundNullishResult = "missing";
  let directBadPredicate = "missing";
  let boundBadPredicateResult = "missing";

  try {
    method.call(null, function() { return true; });
  } catch (error) {
    directNullish =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    boundNullish();
  } catch (error) {
    boundNullishResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    method.call([], null);
  } catch (error) {
    directBadPredicate =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    boundBadPredicate();
  } catch (error) {
    boundBadPredicateResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  results.push(
    names[i] + "=" +
    directNullish + "|" +
    boundNullishResult + "|" +
    directBadPredicate + "|" +
    boundBadPredicateResult
  );
}

results.join(",");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect(
                "borrowed and bound cross-realm Array find-like methods should throw in defining realm",
            );
        assert!(
            outcome.note.contains(
                "string(find=true:true:false|true:true:false|true:true:false|true:true:false,findIndex=true:true:false|true:true:false|true:true:false|true:true:false,findLast=true:true:false|true:true:false|true:true:false|true:true:false,findLastIndex=true:true:false|true:true:false|true:true:false|true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_array_callback_methods_throw_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let names = ["map", "filter", "every", "some"];
let results = [];

for (let i = 0; i < names.length; i++) {
  let method = other.global.Array.prototype[names[i]];
  let boundNullish = method.bind(null, function(value) { return value; });
  let boundBadCallback = method.bind([], null);
  let directNullish = "missing";
  let boundNullishResult = "missing";
  let directBadCallback = "missing";
  let boundBadCallbackResult = "missing";

  try {
    method.call(null, function(value) { return value; });
  } catch (error) {
    directNullish =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    boundNullish();
  } catch (error) {
    boundNullishResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    method.call([], null);
  } catch (error) {
    directBadCallback =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    boundBadCallback();
  } catch (error) {
    boundBadCallbackResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  results.push(
    names[i] + "=" +
    directNullish + "|" +
    boundNullishResult + "|" +
    directBadCallback + "|" +
    boundBadCallbackResult
  );
}

results.join(",");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect(
                "borrowed and bound cross-realm Array callback methods should throw in defining realm",
            );
        assert!(
            outcome.note.contains(
                "string(map=true:true:false|true:true:false|true:true:false|true:true:false,filter=true:true:false|true:true:false|true:true:false|true:true:false,every=true:true:false|true:true:false|true:true:false|true:true:false,some=true:true:false|true:true:false|true:true:false|true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_array_iterator_methods_throw_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let names = ["keys", "entries", "values"];
let results = [];

for (let i = 0; i < names.length; i++) {
  let method = other.global.Array.prototype[names[i]];
  let bound = method.bind(null);
  let direct = "missing";
  let boundResult = "missing";

  try {
    method.call(null);
  } catch (error) {
    direct =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    bound();
  } catch (error) {
    boundResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  results.push(names[i] + "=" + direct + "|" + boundResult);
}

{
  let iterator = other.global.Array.prototype.values.call(new other.global.Array(1));
  let method = Object.getPrototypeOf(iterator).next;
  let bound = method.bind({});
  let direct = "missing";
  let boundResult = "missing";
  let nextResult = "missing";

  try {
    method.call({});
  } catch (error) {
    direct =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    bound();
  } catch (error) {
    boundResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  nextResult = method.call(iterator);
  results.push(
    "next=" + direct + "|" + boundResult + "|" +
    (Object.getPrototypeOf(nextResult) === other.global.Object.prototype) + ":" +
    (Object.getPrototypeOf(nextResult) === Object.prototype)
  );
}

results.join(",");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect(
                "borrowed and bound cross-realm Array iterator methods should throw in defining realm",
            );
        assert!(
            outcome.note.contains(
                "string(keys=true:true:false|true:true:false,entries=true:true:false|true:true:false,values=true:true:false|true:true:false,next=true:true:false|true:true:false|true:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_cross_realm_string_methods_throw_other_realm_type_error() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm();
let names = [
  "at", "charAt", "concat", "endsWith", "includes", "indexOf", "isWellFormed",
  "match", "matchAll", "padEnd", "padStart", "repeat", "replace",
  "replaceAll", "search", "slice", "split", "startsWith", "toUpperCase",
  "toWellFormed", "trim", "trimEnd", "trimStart"
];
let results = [];

for (let i = 0; i < names.length; i++) {
  let method = other.global.String.prototype[names[i]];
  let bound = method.bind(null, "x");
  let direct = "missing";
  let boundResult = "missing";

  try {
    method.call(null, "x");
  } catch (error) {
    direct =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    bound();
  } catch (error) {
    boundResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  results.push(names[i] + "=" + direct + "|" + boundResult);
}

results.join(",");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("borrowed and bound cross-realm String methods should throw in defining realm");
        assert!(
            outcome.note.contains(
                "string(at=true:true:false|true:true:false,charAt=true:true:false|true:true:false,concat=true:true:false|true:true:false,endsWith=true:true:false|true:true:false,includes=true:true:false|true:true:false,indexOf=true:true:false|true:true:false,isWellFormed=true:true:false|true:true:false,match=true:true:false|true:true:false,matchAll=true:true:false|true:true:false,padEnd=true:true:false|true:true:false,padStart=true:true:false|true:true:false,repeat=true:true:false|true:true:false,replace=true:true:false|true:true:false,replaceAll=true:true:false|true:true:false,search=true:true:false|true:true:false,slice=true:true:false|true:true:false,split=true:true:false|true:true:false,startsWith=true:true:false|true:true:false,toUpperCase=true:true:false|true:true:false,toWellFormed=true:true:false|true:true:false,trim=true:true:false|true:true:false,trimEnd=true:true:false|true:true:false,trimStart=true:true:false|true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_string_concat_is_generic_and_orders_string_conversion() {
        let source = r#"
let order = [];
let receiver = {
  toString() {
    order.push("receiver");
    return "base";
  }
};
receiver.concat = String.prototype.concat;
let first = {
  toString() {
    order.push("first");
    return ":first";
  }
};
let second = {
  toString() {
    order.push("second");
    return ":second";
  }
};
receiver.concat(first, second) + "|" + order.join(",");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("String.prototype.concat should be generic and preserve conversion order");
        assert!(
            outcome
                .note
                .contains("string(base:first:second|receiver,first,second)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_string_replace_all_handles_literal_and_function_replacements() {
        let outcome = engine()
            .run_script(
                r#"
let calls = [];
let functional = "a,b,".replaceAll(",", function(match, position, string) {
  calls.push(match + ":" + position + ":" + string);
  return "$&";
});
let substitution = "aba".replaceAll("a", "$`|$&|$'|$$|$1|$<x>");
let empty = "ab".replaceAll("", "-");
let first = "aaa".replace("a", "x");
let regexpCalls = [];
let regexp = /(a)/g[Symbol.replace]("aaa abc", function(match, capture, position, string) {
  regexpCalls.push(match + ":" + capture + ":" + position + ":" + string);
  return "z";
});
[functional, calls.join(";"), substitution, empty, first, regexp, regexpCalls.join(";")].join("\n");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should perform literal string replacements");
        assert!(
            outcome.note.contains(
                "string(a$&b$&\n,:1:a,b,;,:3:a,b,\n|a|ba|$|$1|$<x>bab|a||$|$1|$<x>\n-a-b-\nxaa\nzzz zbc\na:a:0:aaa abc;a:a:1:aaa abc;a:a:2:aaa abc;a:a:4:aaa abc)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_string_split_orders_and_catches_limit_coercion() {
        let source = "var order = []; var separator = { toString: function() { order.push('separator'); throw 'separator'; } }; var limit = { valueOf: function() { order.push('limit'); throw 'limit'; } }; var caught = 'none'; try { 'x y'.split(separator, limit); } catch (error) { caught = error; } var pieces = new String('one two three').split(/ /, 2); caught + '|' + order.join(',') + '|' + pieces.join(',');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("String.prototype.split should preserve coercion order and catches");
        assert!(
            outcome.note.contains("string(limit|limit|one,two)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_string_substring_treats_explicit_undefined_as_string_end() {
        let source = r#"
let caught = "none";
try {
  "abc".substring({ valueOf() { throw "start"; } }, undefined);
} catch (error) {
  caught = error;
}
["undefined".substring("e", undefined), caught].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("String.prototype.substring should handle undefined end and coercion throws");
        assert!(
            outcome.note.contains("string(undefined|start)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_string_from_char_code_handles_variadic_uint16_and_detached_calls() {
        let source = r#"
let fromCharCode = String.fromCharCode;
delete String.fromCharCode;
[
  fromCharCode(),
  fromCharCode(65, 66),
  fromCharCode(Infinity).charCodeAt(0),
  fromCharCode(-1.2).charCodeAt(0),
  fromCharCode.name,
  fromCharCode.length
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("String.fromCharCode should apply ToUint16 in detached variadic calls");
        assert!(
            outcome.note.contains("string(|AB|0|65535|fromCharCode|1)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_string_from_code_point_handles_variadic_unicode_and_abrupt_values() {
        let source = r#"
let failures = [];
try {
  String.fromCodePoint(3.5);
} catch (error) {
  failures.push(error.name);
}
try {
  String.fromCodePoint(65, { valueOf() { throw "coercion"; } });
} catch (error) {
  failures.push(error);
}
[
  String.fromCodePoint(65, 0x1D306),
  String.fromCodePoint(0x2F804),
  String.fromCodePoint.length,
  String.fromCodePoint.name,
  failures.join(",")
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("String.fromCodePoint should encode scalars and preserve abrupt conversions");
        assert!(
            outcome
                .note
                .contains("string(A𝌆|你|1|fromCodePoint|RangeError,coercion)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_string_raw_handles_templates_substitutions_and_abrupt_getters() {
        let source = r#"
let steps = [];
let template = {
  raw: {
    get length() { steps.push("length"); return 2; },
    get 0() { steps.push("first"); return "a"; },
    get 1() { steps.push("second"); throw "segment"; }
  }
};
let substitution = {
  toString() { steps.push("substitution"); return "b"; }
};
let caught = "none";
try {
  String.raw(template, substitution);
} catch (error) {
  caught = error;
}
[
  String.raw({ raw: ["a", "c"] }, substitution),
  String.raw`line\\n${"value"}`,
  String.raw.length,
  String.raw.name,
  steps.join(","),
  caught
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("String.raw should preserve raw segments and abrupt getter ordering");
        assert!(
            outcome.note.contains(
                "string(abc|line\\\\nvalue|1|raw|length,first,substitution,second,substitution,substitution|segment)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_string_normalize_handles_all_forms_hangul_and_coercion() {
        let source = r#"
let decomposedHangul = "\u1100\u1161\u11A8";
[
  "\u1E9B\u0323".normalize("NFD") === "\u017F\u0323\u0307",
  "\uFB01".normalize("NFKC"),
  decomposedHangul.normalize("NFC"),
  "\uAC01".normalize("NFD") === decomposedHangul,
  "\uD800".normalize() === "\uD800",
  "\u00E9".normalize({ toString() { return "NFD"; } })
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("String.prototype.normalize should implement all Unicode normalization forms");
        assert!(
            outcome.note.contains("string(true|fi|각|true|true|é)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_string_locale_compare_handles_canonical_equivalence_and_coercion() {
        let source = r#"
let receiver = {
  toString() { return "o\u0308"; }
};
let that = {
  toString() { return "\u00F6"; }
};
[
  String.prototype.localeCompare.call(receiver, that),
  "a".localeCompare("b"),
  "b".localeCompare("a"),
  "undefined".localeCompare(),
  String.prototype.localeCompare.length,
  String.prototype.localeCompare.name
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("String.prototype.localeCompare should normalize and compare strings");
        assert!(
            outcome.note.contains("string(0|-1|1|0|1|localeCompare)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_string_to_lower_case_handles_unicode_and_coercion_throws() {
        let source = r#"
let receiver = {
  toString() {
    throw "coercion";
  }
};
receiver.toLowerCase = String.prototype.toLowerCase;
let caught = "none";
try {
  receiver.toLowerCase();
} catch (error) {
  caught = error;
}
[
  "ABC".toLowerCase(),
  "\u0130".toLowerCase(),
  "ΟΣ".toLowerCase(),
  "ΟΣΑ".toLowerCase(),
  "A\u180EΣ".toLowerCase(),
  "ΟΣ".toLocaleLowerCase(),
  caught,
  "".toLowerCase().index === undefined
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("String.prototype.toLowerCase should implement Unicode casing and coercion");
        assert!(
            outcome
                .note
                .contains("string(abc|i̇|ος|οσα|a᠎ς|ος|coercion|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_string_to_upper_case_handles_unicode_and_coercion_throws() {
        let source = r#"
let receiver = {
  toString() {
    throw "coercion";
  }
};
receiver.toUpperCase = String.prototype.toUpperCase;
let caught = "none";
try {
  receiver.toUpperCase();
} catch (error) {
  caught = error;
}
[
  "straße".toUpperCase(),
  "\u0390".toUpperCase(),
  "\uD801\uDC28".toUpperCase(),
  "straße".toLocaleUpperCase(),
  caught
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("String.prototype.toUpperCase should implement Unicode casing and coercion");
        assert!(
            outcome
                .note
                .contains("string(STRASSE|Ϊ́|𐐀|STRASSE|coercion)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_create_realm_uses_realm_local_string_constructor_and_prototype() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let value = new other.String("abc");
let thisValue = new String("abc");
[
  other.String === String,
  other.String.prototype === String.prototype,
  Object.getPrototypeOf(other.String) === other.Function.prototype,
  Object.getPrototypeOf(other.String.prototype) === other.Object.prototype,
  other.String.prototype.constructor === other.String,
  Object.getPrototypeOf(value) === other.String.prototype,
  value instanceof other.String,
  value instanceof String,
  typeof other.String.prototype.toString,
  typeof other.String.prototype.valueOf,
  typeof other.String.prototype.charAt,
  other.String.prototype.toString.call(thisValue),
  other.String.prototype.valueOf.call(thisValue),
  other.String.prototype.charAt.call(thisValue, 1)
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm should use realm-local String constructor and prototype");
        assert!(
            outcome.note.contains(
                "string(false|false|true|true|true|true|true|false|function|function|function|abc|abc|b)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_proxy_get_null_trap_forwards_to_proxy_target_get() {
        let outcome = engine()
            .run_script(
                r#"
let stringTarget = new Proxy(new String("str"), {});
let stringProxy = new Proxy(stringTarget, { get: null });
let sym = Symbol();
let target = new Proxy({}, {
  get: function(_target, key) {
    switch (key) {
      case sym: return 1;
      case "10": return 2;
      case "foo": return 3;
    }
  },
});
let proxy = new Proxy(target, { get: null });
(stringProxy[0] === "s") + ":" + (proxy[sym] === 1) + ":" + (proxy[10] === 2) + ":" + (Object.create(proxy).foo === 3);
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("null proxy get trap should forward to proxy target get");
        assert!(
            outcome.note.contains("string(true:true:true:true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_proxy_get_enforces_non_configurable_target_invariants() {
        for source in [
            "let target = {}; Object.defineProperty(target, 'attr', { configurable: false, writable: false, value: 1 }); let p = new Proxy(target, { get() { return 2; } }); p.attr;",
            "let target = {}; Object.defineProperty(target, 'attr', { configurable: false, get: undefined }); let p = new Proxy(target, { get() { return 2; } }); p.attr;",
        ] {
            let err = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_err();
            assert!(
                err.message().contains("uncaught throw: TypeError"),
                "source: {source}, error: {err:?}"
            );
        }
    }

    #[test]
    fn wasm_backend_runtime_throws_for_non_callable_method_and_reads_array_length_brackets() {
        let method_err = engine()
            .run_script(
                "let obj = { f: 1 }; obj.f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("non-callable method should throw");
        assert!(method_err.message().contains("uncaught throw: TypeError"));

        let length_outcome = engine()
            .run_script(
                "let a = [1]; a[\"length\"];",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("array length bracket should run");
        assert!(length_outcome.note.contains("number(1)"));
    }

    #[test]
    fn wasm_backend_supports_script_closure_capture() {
        let outcome = engine()
            .run_script(
                "let x = 1; function f() { return x; } f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("script closure should run");
        assert!(outcome.note.contains("number(1"));
    }

    #[test]
    fn wasm_backend_supports_nested_function_declaration() {
        let outcome = engine()
            .run_script(
                "function outer() { let x = 1; function inner() { return x + 1; } return inner(); } outer();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("nested function declaration should run");
        assert!(outcome.note.contains("number(2"));
    }

    #[test]
    fn wasm_backend_supports_closure_mutation() {
        let outcome = engine()
            .run_script(
                "function outer() { let x = 1; function inc() { x = x + 1; return x; } inc(); return inc(); } outer();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("closure mutation should run");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_numeric_update_coerces_captured_boolean_binding() {
        let outcome = engine()
            .run_script(
                "let value = true; function increment() { return ++value; } increment();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("numeric update should coerce the captured boolean binding");
        assert!(outcome.note.contains("number(2"), "note: {}", outcome.note);
    }

    #[test]
    fn wasm_backend_numeric_update_preserves_dynamic_bigint() {
        let outcome = engine()
            .run_script(
                "function increment(value) { let previous = value++; return previous === 4n && value === 5n; } increment({ value: 4n }.value);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("numeric update should preserve a dynamic BigInt binding");
        assert!(
            outcome.note.contains("boolean(true"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_supports_anonymous_function_expression() {
        let outcome = engine()
            .run_script(
                "let f = function (x) { return x + 1; }; f(2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("anonymous function expression should run");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_returned_closure_call() {
        let outcome = engine()
            .run_script(
                "function outer() { let x = 2; return function (y) { return x + y; }; } let f = outer(); f(3);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("returned closure should run");
        assert!(outcome.note.contains("number(5"));
    }

    #[test]
    fn wasm_backend_supports_object_closure_method() {
        let outcome = engine()
            .run_script(
                "function outer() { let x = 3; return { f: function () { return x; } }; } let o = outer(); o.f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("object closure method should run");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_var_closure_capture() {
        let outcome = engine()
            .run_script(
                "function outer() { var x = 1; return function () { return x; }; } let f = outer(); f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("var closure should run");
        assert!(outcome.note.contains("number(1"));
    }

    #[test]
    fn wasm_backend_supports_nested_recursive_closure_call() {
        let outcome = engine()
            .run_script(
                "function outer(n) { function loop(x) { if (x === 0) { return 0; } return loop(x - 1) + 1; } return loop(n); } outer(3);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("nested recursive function should run");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_arrow_function_basic() {
        let outcome = engine()
            .run_script(
                "let f = x => x + 1; f(2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("arrow function should run");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_arrow_function_block_body() {
        let outcome = engine()
            .run_script(
                "let f = x => { return x + 1; }; f(2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("arrow block body should run");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_named_function_expression_recursion() {
        let outcome = engine()
            .run_script(
                "let f = function fact(n) { if (n === 0) { return 1; } return n * fact(n - 1); }; f(4);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("named function expression should run");
        assert!(outcome.note.contains("number(24"));
    }

    #[test]
    fn wasm_backend_supports_returned_arrow_closure_call() {
        let outcome = engine()
            .run_script(
                "function outer(x) { return y => x + y; } let f = outer(2); f(3);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("returned arrow closure should run");
        assert!(outcome.note.contains("number(5"));
    }

    #[test]
    fn wasm_backend_supports_arrow_lexical_this() {
        let outcome = engine()
            .run_script(
                "function make() { return () => this.x; } let o = { x: 3, f: make }; let g = o.f(); g();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("arrow lexical this should run");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_object_literal_shorthand_methods_and_accessors() {
        let shorthand_outcome = engine()
            .run_script(
                "let x = 1; let o = { x }; o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("object shorthand should run");
        assert!(shorthand_outcome.note.contains("number(1"));

        let method_outcome = engine()
            .run_script(
                "let o = { x: 3, f() { return this.x; } }; o.f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("concise method should run");
        assert!(method_outcome.note.contains("number(3"));

        let closure_method_outcome = engine()
            .run_script(
                "function make(x) { return { f() { return x; } }; } make(2).f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("closure method should run");
        assert!(closure_method_outcome.note.contains("number(2"));

        let getter_outcome = engine()
            .run_script(
                "let o = { get x() { return 1; } }; o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("getter should run");
        assert!(getter_outcome.note.contains("number(1"));

        let setter_outcome = engine()
            .run_script(
                "let o = { _x: 0, set x(v) { this._x = v; } }; o.x = 3; o._x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("setter should run");
        assert!(setter_outcome.note.contains("number(3"));

        let pair_outcome = engine()
            .run_script(
                "let o = { _x: 0, get x() { return this._x; }, set x(v) { this._x = v; } }; o.x = 4; o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("getter setter pair should run");
        assert!(pair_outcome.note.contains("number(4"));

        let arrow_method_outcome = engine()
            .run_script(
                "let o = { x: 3, f() { return (() => this.x)(); } }; o.f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("arrow in method should keep lexical this");
        assert!(arrow_method_outcome.note.contains("number(3"));

        let returned_accessor_outcome = engine()
            .run_script(
                "function make() { return { get x() { return 5; } }; } let o = make(); o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("returned accessor object should run");
        assert!(returned_accessor_outcome.note.contains("number(5"));
    }

    #[test]
    fn wasm_backend_sloppy_assignment_ignores_inherited_accessor_without_setter() {
        let outcome = engine()
            .run_script(
                r#"
let prototype = {};
Object.defineProperty(prototype, "value", {
  get: function() { return 7; }
});
let object = Object.create(prototype);
let before = object.value;
let prototypeMatches = Object.getPrototypeOf(object) === prototype;
object.value = 9;
before + "|" + prototypeMatches + "|" +
  Object.prototype.hasOwnProperty.call(object, "value") + "|" + object.value;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("sloppy assignment should ignore an inherited accessor without a setter");
        assert!(
            outcome.note.contains("string(7|true|false|7)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_strict_assignment_to_inherited_accessor_without_setter_throws() {
        let outcome = engine()
            .run_script(
                r#"
let prototype = {};
Object.defineProperty(prototype, "value", {
  get: function() { return 7; }
});
let object = Object.create(prototype);
let caught = false;
try {
  (function() { "use strict"; object.value = 9; })();
} catch (error) {
  caught = error instanceof TypeError;
}
caught + "|" + Object.prototype.hasOwnProperty.call(object, "value") + "|" + object.value;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("strict assignment to an inherited accessor without a setter should throw");
        assert!(
            outcome.note.contains("string(true|false|7)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_sloppy_assignment_ignores_inherited_read_only_property() {
        let outcome = engine()
            .run_script(
                r#"
let prototype = {};
Object.defineProperty(prototype, "value", { value: 7, writable: false });
let object = Object.create(prototype);
object.value = 9;
Object.prototype.hasOwnProperty.call(object, "value") + "|" + object.value;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("sloppy assignment should ignore an inherited read-only property");
        assert!(
            outcome.note.contains("string(false|7)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_strict_assignment_to_inherited_read_only_property_throws() {
        let outcome = engine()
            .run_script(
                r#"
let prototype = {};
Object.defineProperty(prototype, "value", { value: 7, writable: false });
let object = Object.create(prototype);
let caught = false;
try {
  (function() { "use strict"; object.value = 9; })();
} catch (error) {
  caught = error instanceof TypeError;
}
caught + "|" + Object.prototype.hasOwnProperty.call(object, "value") + "|" + object.value;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("strict assignment to an inherited read-only property should throw");
        assert!(
            outcome.note.contains("string(true|false|7)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_assignment_shadows_inherited_writable_property() {
        let outcome = engine()
            .run_script(
                r#"
let prototype = { value: 7 };
let object = Object.create(prototype);
object.value = 9;
Object.prototype.hasOwnProperty.call(object, "value") + "|" +
  object.value + "|" + prototype.value;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("assignment should shadow an inherited writable property");
        assert!(
            outcome.note.contains("string(true|9|7)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_supports_script_global_object_core() {
        let top_level_this = engine()
            .run_script(
                "this === globalThis;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("top-level this should run");
        assert!(top_level_this.note.contains("boolean(true"));

        let global_var = engine()
            .run_script(
                "{ var x = 1; } globalThis.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("global var alias should run");
        assert!(global_var.note.contains("number(1"));

        let lexical_not_global = engine()
            .run_script(
                "let x = 1; globalThis.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("top-level lexical should stay off global object");
        assert!(lexical_not_global.note.contains("undefined(undefined"));

        let lexical_not_own_property = engine()
            .run_script(
                "let x = 1; \"x\" in globalThis;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("top-level lexical should not define global object property");
        assert!(lexical_not_own_property.note.contains("boolean(false"));

        let default_this = engine()
            .run_script(
                "function f() { return this; } f() === globalThis;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("bare call default this should run");
        assert!(default_this.note.contains("boolean(true"));

        let lexical_this = engine()
            .run_script(
                "let f = () => this; f() === globalThis;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("top-level arrow lexical this should run");
        assert!(lexical_this.note.contains("boolean(true"));

        let global_function = engine()
            .run_script(
                "function f() {} globalThis.f;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("global function property should run");
        assert!(global_function.note.contains("function(handle@"));
    }

    #[test]
    fn wasm_backend_supports_default_rest_and_arguments_core() {
        for (source, expected, label) in [
            (
                "function f(x = 1) { return x; } f();",
                "number(1",
                "default param basic",
            ),
            (
                "function f(x, y = x + 1) { return y; } f(2);",
                "number(3",
                "default param from earlier param",
            ),
            (
                "let f = (x = 1) => x + 1; f();",
                "number(2",
                "arrow default param",
            ),
            (
                "function third(...xs) { return xs[2]; } third(1, 2, 3);",
                "number(3",
                "rest param element",
            ),
            (
                "function len(...xs) { return xs.length; } len(1, 2, 3);",
                "number(3",
                "rest param length",
            ),
            (
                "function f(a, b) { return arguments.length; } f(1, 2, 3);",
                "number(3",
                "arguments length",
            ),
            (
                "function f() { return arguments[1]; } f(1, 2, 3);",
                "number(2",
                "arguments indexed read",
            ),
            (
                "function f(x) { arguments[0] = 3; return x; } f(1);",
                "number(3",
                "mapped arguments write to param",
            ),
            (
                "function f(x) { x = 4; return arguments[0]; } f(1);",
                "number(4",
                "mapped param write to arguments",
            ),
            (
                "function f(x, x, x) { var before = arguments[0] + ',' + arguments[1] + ',' + arguments[2]; x = 4; return before + '|' + arguments[0] + ',' + arguments[1] + ',' + arguments[2]; } f(1, 2, 3);",
                "string(1,2,3|1,2,4)",
                "only the last duplicate parameter maps to arguments",
            ),
            (
                "function f(x = 1) { arguments[0] = 3; return x; } f(2);",
                "number(2",
                "unmapped default param arguments",
            ),
            (
                "function f(...xs) { arguments[0] = 9; return xs[0]; } f(1, 2);",
                "number(1",
                "unmapped rest arguments",
            ),
            (
                "function outer() { return (() => arguments[0])(); } outer(3);",
                "number(3",
                "arrow lexical arguments",
            ),
            (
                "let o = { x: 2, f(y = this.x) { return y; } }; o.f();",
                "number(2",
                "method default with this",
            ),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|_| panic!("{label} should run"));
            assert!(
                outcome.note.contains(expected),
                "{label} produced unexpected note: {}",
                outcome.note
            );
        }
    }

    #[test]
    fn wasm_backend_rejects_unsupported_param_and_arguments_forms() {
        for source in [
            "function f(x = y, y = 1) { return x; } f();",
            "function f(x = x) { return x; } f();",
            "({ get x(a) { return a; } }).x;",
            "({ set x(v = 1) {} }).x = 1;",
        ] {
            let err = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect_err("unsupported param or arguments form should stay unsupported");
            assert!(!err.message().trim().is_empty());
        }
    }

    #[test]
    fn wasm_backend_supports_arguments_callee_identity() {
        let outcome = engine()
            .run_script(
                "function f() { return arguments.callee === f; } f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("arguments.callee should run");

        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_supports_host_print_global() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        for source in [
            "print(\"grug\")",
            "globalThis.print(\"grug\")",
            "let p = print; p(\"grug\")",
            "let o = { f: print }; o.f(\"grug\")",
            "function f() { print(\"x\"); } f()",
        ] {
            let outcome = engine_with_captured_prints(Arc::clone(&lines))
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect("host print should run");
            assert!(
                outcome.note.contains("undefined"),
                "source: {source}, note: {}",
                outcome.note
            );
        }
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "grug".to_string(),
                "grug".to_string(),
                "grug".to_string(),
                "grug".to_string(),
                "x".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_supports_host_print_from_shared_memory_module() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let outcome = engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                "const values = new Int32Array(new SharedArrayBuffer(4)); values[0] = 7; const old = Atomics.add(values, 0, 5); print(old + ':' + values[0]);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("host print should read strings from shared Wasm memory");
        assert!(outcome.note.contains("undefined"), "{}", outcome.note);
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["7:12".to_string()]
        );
    }

    #[test]
    fn wasm_backend_rejects_remaining_global_object_tails() {
        let err = engine()
            .run_script(
                "arguments",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("unsupported global seam should stay unsupported");
        let message = err.message();
        assert!(
            message.contains("unsupported in porffor wasm-aot first slice")
                || message.contains("ReferenceError"),
            "err: {message}"
        );
    }

    #[test]
    fn wasm_backend_supports_sloppy_global_name_resolution() {
        for (source, expected, label) in [
            (
                "globalThis.x = 1; x;",
                "number(1",
                "read after globalThis write",
            ),
            (
                "missing = 1; globalThis.missing;",
                "number(1",
                "implicit global create",
            ),
            (
                "function f() { return x; } globalThis.x = 2; f();",
                "number(2",
                "function global read",
            ),
            (
                "function f() { y = 3; } f(); globalThis.y;",
                "number(3",
                "function implicit global write",
            ),
            (
                "let x = 1; globalThis.x = 2; x;",
                "number(1",
                "lexical shadows global",
            ),
            (
                "function f() { return () => z; } z = 4; f()();",
                "number(4",
                "closure global read",
            ),
            ("x = 1; x++; x;", "number(2", "global numeric update"),
            (
                "globalThis.x = 1; x += 2; x;",
                "number(3",
                "global compound assign",
            ),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|_| panic!("{label} should run"));
            assert!(
                outcome.note.contains(expected),
                "{label} produced unexpected note: {}",
                outcome.note
            );
        }
    }

    #[test]
    fn wasm_backend_rejects_remaining_sloppy_global_tails() {
        for source in [
            "x",
            "function f() { return q; } f()",
            "if (true) { globalThis.x = 1; } else {} x",
            "topLevel = arguments",
        ] {
            let err = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect_err("unsupported sloppy global seam should stay unsupported");
            let message = err.message();
            assert!(
                message.contains("unsupported in porffor wasm-aot first slice")
                    || message.contains("ReferenceError"),
                "source: {source}, err: {message}"
            );
        }
    }

    #[test]
    fn wasm_backend_rejects_unsupported_object_literal_method_forms() {
        for source in [
            "({ get x(v) { return v; } })",
            "({ set x() {} })",
            "({ f() { return super.x; } })",
        ] {
            let err = match engine().run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            ) {
                Ok(outcome) => panic!(
                    "unsupported object literal form should stay unsupported for `{source}`: {outcome:?}"
                ),
                Err(err) => err,
            };
            assert!(!err.message().trim().is_empty());
        }
    }

    #[test]
    fn wasm_backend_supports_implicit_undefined_function_return() {
        let outcome = engine()
            .run_script(
                "function f() { let x = 1; } f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run implicit undefined function");
        assert!(outcome.note.contains("undefined("));
    }

    #[test]
    fn wasm_backend_supports_while_loop() {
        let outcome = engine()
            .run_script(
                "let i = 0; while (i < 3) { i = i + 1; } i;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run while loop");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_do_while_loop() {
        let outcome = engine()
            .run_script(
                "let i = 0; do { i = i + 1; } while (i < 3); i;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run do while loop");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_for_break_and_continue() {
        let outcome = engine()
            .run_script(
                "let i = 0; let sum = 0; for (; i < 5; i = i + 1) { if (i === 2) { continue; } if (i === 4) { break; } sum = sum + i; } sum;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run for loop");
        assert!(outcome.note.contains("number(4"));
    }

    #[test]
    fn wasm_backend_supports_update_and_compound_assignment() {
        let outcome = engine()
            .run_script(
                "let sum = 0; for (let i = 0; i < 4; i++) { sum += i; } sum;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run updates and compound assignment");
        assert!(outcome.note.contains("number(6"));
    }

    #[test]
    fn wasm_backend_preserves_postfix_result() {
        let outcome = engine()
            .run_script(
                "let i = 2; let x = i++; x + i;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should preserve postfix value");
        assert!(outcome.note.contains("number(5"));
    }

    #[test]
    fn wasm_backend_supports_switch_and_labels() {
        let outcome = engine()
            .run_script(
                "let x = 0; outer: while (x < 3) { x += 1; switch (x) { case 1: continue outer; case 2: debugger; break outer; default: x = 9; } } x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run switch and labels");
        assert!(outcome.note.contains("number(2"));
    }

    #[test]
    fn wasm_backend_supports_default_in_middle_switch() {
        let outcome = engine()
            .run_script(
                "let x = 0; switch (3) { case 1: x = 1; break; default: x = 9; break; case 3: x = 3; } x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run default-in-middle switch");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_rejects_const_update_precisely() {
        let err = engine()
            .run_script(
                "const x = 1; x++;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("const update should stay unsupported");
        assert!(err
            .message()
            .contains("unsupported in porffor wasm-aot first slice: update of const binding"));
    }

    #[test]
    fn wasm_backend_supports_label_on_non_loop_statement() {
        let outcome = engine()
            .run_script(
                "let value = 0; label: if (true) { value = 1; break label; value = 2; } value;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("labels should target any statement");
        assert!(outcome.note.contains("number(1"), "note: {}", outcome.note);
    }

    #[test]
    fn wasm_backend_supports_hoisted_var() {
        let outcome = engine()
            .run_script(
                "x; var x = 1;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should support hoisted var");
        assert!(outcome.note.contains("undefined("));
    }

    #[test]
    fn wasm_backend_supports_var_in_for_and_duplicate_var() {
        let outcome = engine()
            .run_script(
                "var sum = 0; for (var i = 0; i < 4; i++) { sum += i; } var sum; sum;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should support var in for");
        assert!(outcome.note.contains("number(6"));
    }

    #[test]
    fn wasm_backend_supports_dynamic_primitive_string_concat_and_equality() {
        for (source, expected) in [
            ("\"a\" + \"b\";", "string(ab)"),
            ("\"x\" + 1;", "string(x1)"),
            ("function f(x) { return \"v=\" + x; } f(3);", "string(v=3)"),
            ("(\"a\" + \"b\") === \"ab\";", "boolean(true)"),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect("string concat or equality should run");
            assert!(outcome.note.contains(expected));
        }
    }

    #[test]
    fn wasm_backend_supports_mixed_logical_and_nullish() {
        for (source, expected) in [
            ("let x = 0; x || \"fallback\";", "string(fallback)"),
            ("let x = 1; x || \"fallback\";", "number(1"),
            ("let x = null; x ?? 3;", "number(3"),
            ("let x = 0; x ?? 3;", "number(0"),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect("logical or nullish op should run");
            assert!(outcome.note.contains(expected));
        }
    }

    #[test]
    fn wasm_backend_supports_typeof_core() {
        for (source, expected) in [
            ("typeof 1;", "string(number)"),
            ("typeof \"x\";", "string(string)"),
            ("typeof undefined;", "string(undefined)"),
            ("function f() {} typeof f;", "string(function)"),
            ("typeof missingName;", "string(undefined)"),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect("typeof should run");
            assert!(outcome.note.contains(expected));
        }
    }

    #[test]
    fn wasm_backend_supports_primitive_coercion_core() {
        for (source, expected) in [
            ("1 == \"1\";", "boolean(true)"),
            ("0 == false;", "boolean(true)"),
            ("null == undefined;", "boolean(true)"),
            ("1 != \"2\";", "boolean(true)"),
            ("\"2\" - 1;", "number(1"),
            ("true + 2;", "number(3"),
            ("null + 1;", "number(1"),
            ("\"6\" / \"2\";", "number(3"),
            ("\"10\" > \"2\";", "boolean(false)"),
            ("\"2\" < 3;", "boolean(true)"),
            ("void 1;", "undefined"),
            ("let x = 1; void (x = 3); x;", "number(3"),
            ("(1, 2);", "number(2"),
            ("let x = 0; (x = 1, x + 2);", "number(3"),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect("primitive coercion core should run");
            assert!(
                outcome.note.contains(expected),
                "source: {source}, note: {}",
                outcome.note
            );
        }
    }

    #[test]
    fn wasm_backend_supports_heap_coercion_core() {
        for (source, expected) in [
            ("\"a\" + {};", "string(a[object Object])"),
            ("let o = {}; o + \"x\";", "string([object Object]x)"),
            ("let o = { valueOf() { return 2; } }; o + 1;", "number(3"),
            (
                "let o = { toString() { return \"x\"; } }; o + 1;",
                "string(x1)",
            ),
            ("[] + 1;", "string(1)"),
            ("[1, 2] + 3;", "string(1,23)"),
            ("let o = {}; o == \"[object Object]\";", "boolean(true)"),
            (
                "let o = { valueOf() { return 2; } }; o == \"2\";",
                "boolean(true)",
            ),
            ("let o = {}; o == o;", "boolean(true)"),
            ("[2] < 3;", "boolean(true)"),
            (
                "function f() { return arguments + \"\"; } f(1, 2);",
                "string([object Arguments])",
            ),
            ("let f = function() {}; \"x\" + f;", "string(xfunction("),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect("heap coercion should run");
            assert!(
                outcome.note.contains(expected),
                "source: {source}, note: {}",
                outcome.note
            );
        }
    }

    #[test]
    fn wasm_backend_rejects_remaining_out_of_slice_heap_coercions() {
        for source in ["let o = { toString() { return function() {}; } }; \"\" + o;"] {
            let err = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect_err("out-of-slice dynamic primitive op should stay unsupported");
            assert!(!err.message().trim().is_empty(), "source: {source}");
        }
    }

    #[test]
    fn wasm_backend_supports_new_and_instanceof_core() {
        for (source, expected) in [
            (
                "function F() {} let x = new F(); x instanceof F;",
                "boolean(true)",
            ),
            (
                "function F() { this.x = 3; } let x = new F(); x.x;",
                "number(3)",
            ),
            (
                "function F() {} F.prototype.getX = function () { return this.x; }; let x = new F(); x.x = 4; x.getX();",
                "number(4)",
            ),
            (
                "function F() {} F.prototype = { x: 7 }; let x = new F(); x.x;",
                "number(7)",
            ),
            (
                "function F() { this.x = 1; return 2; } let x = new F(); x.x;",
                "number(1)",
            ),
            (
                "function F() { this.x = 1; return { y: 2 }; } let x = new F(); x.y;",
                "number(2)",
            ),
            (
                "function make(v) { return function F() { this.x = v; }; } let F = make(5); let x = new F(); x.x;",
                "number(5)",
            ),
            (
                "function F() {} function G() {} let x = new F(); x instanceof G;",
                "boolean(false)",
            ),
            (
                "class C { constructor() { this.x = 1; } } let c = new C(); c.x;",
                "number(1)",
            ),
            (
                "let C = class { constructor(v) { this.x = v; } }; new C(2).x;",
                "number(2)",
            ),
            (
                "class C { x = 1; static y = 2; } let c = new C(); c.x + C.y;",
                "number(3)",
            ),
            (
                "class C { static x = 1; static { this.y = this.x + 1; } } C.y;",
                "number(2)",
            ),
            (
                "class C { m() { return 1; } } new C().m();",
                "number(1)",
            ),
            (
                "class C { get x() { return 3; } } new C().x;",
                "number(3)",
            ),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| panic!("constructor core should run for `{source}`: {err:?}"));
            assert!(outcome.note.contains(expected), "source: {source}, note: {}", outcome.note);
        }
    }

    #[test]
    fn wasm_backend_rejects_non_constructable_new_and_instanceof_tails() {
        for source in [
            "new.target;",
            "function F() {} let rhs; if (true) { rhs = F; } else { rhs = print; } ({} instanceof rhs);",
        ] {
            let err = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect_err("unsupported constructor edge should stay unsupported");
            let message = err.message();
            assert!(
                message.contains("unsupported in porffor wasm-aot first slice")
                    || message.contains("parse error")
                    || message.contains("TypeError"),
                "source: {source}, err: {message}"
            );
        }
    }

    #[test]
    fn wasm_backend_runtime_throws_for_non_constructable_new_and_instanceof_tails() {
        for source in [
            "try { new (() => 1)(); } catch (e) { e.name; }",
            "try { let o = { f() {} }; new o.f(); } catch (e) { e.name; }",
            "try { let o = { get x() { return 1; } }; new o.x(); } catch (e) { e.name; }",
            "try { new print(); } catch (e) { e.name; }",
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| {
                    panic!("non-constructable runtime case should run for `{source}`: {err:?}")
                });
            assert!(
                outcome.note.contains("string(TypeError)"),
                "source: {source}, note: {}",
                outcome.note
            );
        }
    }

    #[test]
    fn wasm_backend_supports_class_inheritance_and_private_core() {
        for (source, expected) in [
            (
                "class A { constructor(v) { this.x = v; } } class B extends A { constructor() { super(3); } } new B().x;",
                "number(3)",
            ),
            (
                "class C { #x = 1; getX() { return this.#x; } } new C().getX();",
                "number(1)",
            ),
            (
                "class C { #m() { return 2; } getX() { return this.#m(); } } new C().getX();",
                "number(2)",
            ),
            (
                "class C { get #x() { return 3; } read() { return this.#x; } } new C().read();",
                "number(3)",
            ),
            (
                "class C { static #x = 4; static read() { return C.#x; } } C.read();",
                "number(4)",
            ),
            (
                "class C { #x; static has(obj) { return #x in obj; } } let c = new C(); C.has(c);",
                "boolean(true)",
            ),
            (
                "class A { m() { return 1; } } class B extends A { m() { return super.m() + 1; } } new B().m();",
                "number(2)",
            ),
            (
                "let C = class Self { static make() { return new Self(); } }; C.make() instanceof C;",
                "boolean(true)",
            ),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| panic!("phase 22B class case should run for `{source}`: {err:?}"));
            assert!(outcome.note.contains(expected), "source: {source}, note: {}", outcome.note);
        }
    }

    #[test]
    fn wasm_backend_preserves_array_subclass_prototype_chain() {
        let outcome = engine()
            .run_script(
                r#"
class Sub extends Array {}
let array = new Sub(42, "foo");
let pushedLength = array.push(true);
let empty = new Sub();
let sized = new Sub(7);
[
  Object.getPrototypeOf(array) === Sub.prototype,
  Object.getPrototypeOf(Sub.prototype) === Array.prototype,
  array.constructor === Sub,
  array[0],
  array[1],
  pushedLength,
  array.length,
  array[2],
  empty.length,
  sized.length,
  array instanceof Sub,
  array instanceof Array,
  Sub[Symbol.species] === Sub
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array subclasses should preserve their exotic prototype chain");
        assert!(
            outcome
                .note
                .contains("string(true|true|true|42|foo|3|3|true|0|7|true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_supports_phase_twenty_three_exception_core() {
        for (source, expected) in [
            (
                "var x = 0; try { x = 1; } catch (e) { x = 2; } x;",
                "number(1)",
            ),
            ("try { throw 1; } catch (e) { e; }", "number(1)"),
            (
                "var x; try { throw 1; } catch (e) { x = e; } x;",
                "number(1)",
            ),
            (
                "try { throw \"x\"; } catch (e) { e === \"x\"; }",
                "boolean(true)",
            ),
            (
                "try { throw undefined; } catch (e) { e; }",
                "undefined(undefined)",
            ),
            (
                "var arr = []; Object.defineProperty(arr, \"1\", { get: function() { return 42; }, configurable: true }); [arr[1]].join(\"|\");",
                "string(42)",
            ),
            (
                "var arr = []; Object.defineProperty(arr, \"1\", { get: function() { throw new RangeError(\"x\"); }, configurable: true }); try { arr[1]; } catch (e) { e.name; }",
                "string(RangeError)",
            ),
            (
                "var arr = []; Object.defineProperty(arr, \"1\", { get: function() { throw new RangeError(\"x\"); }, configurable: true }); try { arr.map(function(v) { return v; }); } catch (e) { e.name; }",
                "string(RangeError)",
            ),
            (
                "var obj = { 0: 11, length: 2 }; Object.defineProperty(obj, \"1\", { get: function() { throw new RangeError(\"x\"); }, configurable: true }); try { Array.prototype.map.call(obj, function(v) { return v; }); } catch (e) { e.name; }",
                "string(RangeError)",
            ),
            (
                "try { let x = 1; class C {} throw x; } catch (e) { e; }",
                "number(1)",
            ),
            (
                "try { let x = 1; class C { constructor() { this.x = 2; } } throw new C().x; } catch (e) { e; }",
                "number(2)",
            ),
            (
                "try { if (true) { let x = 3; class C {} throw x; } } catch (e) { e; }",
                "number(3)",
            ),
            (
                "try { label: { if (true) { class C {} throw 4; } } } catch (e) { e; }",
                "number(4)",
            ),
            (
                "try { while (true) { class C {} throw 5; } } catch (e) { e; }",
                "number(5)",
            ),
            (
                "try { do { let x = 7; class C {} throw x; } while (false); } catch (e) { e; }",
                "number(7)",
            ),
            (
                "try { for (let i = 0; i < 1; i = i + 1) { let x = 8; class C {} throw x; } } catch (e) { e; }",
                "number(8)",
            ),
            (
                "try { for (let x of [9]) { class C {} throw x; } } catch (e) { e; }",
                "number(9)",
            ),
            (
                "try { switch (1) { case 1: class C {} throw 6; } } catch (e) { e; }",
                "number(6)",
            ),
            (
                "try { class C {} C(); } catch (e) { e.name; }",
                "string(TypeError)",
            ),
            (
                "try { class C { #x = 1; read(obj) { return obj.#x; } } new C().read({}); } catch (e) { e.name; }",
                "string(TypeError)",
            ),
            (
                "class A {} class B extends A { constructor() { return { ok: 1 }; } } new B().ok;",
                "number(1)",
            ),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| panic!("phase 23 case should run for `{source}`: {err:?}"));
            assert!(outcome.note.contains(expected), "source: {source}, note: {}", outcome.note);
        }
    }

    #[test]
    fn wasm_backend_supports_phase_twenty_four_abrupt_core() {
        for (source, expected) in [
            ("try { 1; } finally {}", "number(1)"),
            ("function f() { try { return 1; } finally {} } f();", "number(1)"),
            (
                "var x = 0; try { throw 1; } catch (e) { x = 1; } finally { x = x + 1; } x;",
                "number(2)",
            ),
            (
                "var x = 0; try { let a = 1; class C {} x = a; } finally { let b = 2; class D {} x = x + b; } x;",
                "number(3)",
            ),
            (
                "var x = 0; try { throw 1; } catch (e) { let a = e; class C {} x = a; } finally { let b = 2; class D {} x = x + b; } x;",
                "number(3)",
            ),
            (
                "function f() { let x = 0; while (true) { try { break; } finally { x = 1; } } return x; } f();",
                "number(1)",
            ),
            ("let o = { x: 1 }; delete o.x; \"x\" in o;", "boolean(false)"),
            ("let a = [1, 2]; delete a[0]; (0 in a);", "boolean(false)"),
            ("let a = [1, 2]; delete a[0]; a.length;", "number(2)"),
            ("\"x\" in { x: 1 }", "boolean(true)"),
            ("function F() {} \"prototype\" in F;", "boolean(true)"),
            ("function f() { return new.target; } f();", "undefined(undefined)"),
            (
                "function F() { this.kind = typeof new.target; this.arrowKind = (() => typeof new.target)(); } let x = new F(); x.kind === \"function\" && x.arrowKind === \"function\";",
                "boolean(true)",
            ),
            (
                "class A { constructor() { this.kind = typeof new.target; } } class B extends A { constructor() { super(); } } new B().kind;",
                "string(function)",
            ),
            (
                "var x; try { \"x\" in 1; } catch (e) { x = e.name; } x;",
                "string(TypeError)",
            ),
            (
                "try { class A {} class B extends A { constructor() { this.x = 1; super(); } } new B(); } catch (e) { e.name; }",
                "string(ReferenceError)",
            ),
            (
                "try { class A {} class B extends A { constructor() { this.x = 1; super(); } } new B(); } catch (e) { e; }",
                "object(handle@",
            ),
            (
                "try { class A {} class B extends A { constructor() {} } new B(); } catch (e) { e.name; }",
                "string(ReferenceError)",
            ),
            (
                "try { class A {} class B extends A { constructor() {} } new B(); } catch (e) { e; }",
                "object(handle@",
            ),
            (
                "try { let marker = 1; class A {} class B extends A { constructor() {} } new B(); marker; } catch (e) { e.name; }",
                "string(ReferenceError)",
            ),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| panic!("phase 24 case should run for `{source}`: {err:?}"));
            assert!(outcome.note.contains(expected), "source: {source}, note: {}", outcome.note);
        }
    }

    #[test]
    fn wasm_backend_supports_phase_thirty_null_heritage_classes() {
        for (source, expected) in [
            (
                "class C extends null { constructor() { return Object.create(new.target.prototype); } } let x = new C(); Object.getPrototypeOf(x) === C.prototype;",
                "boolean(true)",
            ),
            (
                "class C extends null { constructor() { return Object.create(new.target.prototype); } } new C() instanceof C;",
                "boolean(true)",
            ),
            (
                "class C extends null { constructor() { return Object.create(new.target.prototype); } } Object.getPrototypeOf(C.prototype) === null;",
                "boolean(true)",
            ),
            (
                "class C extends null { m() { return 1; } constructor() { return Object.create(new.target.prototype); } } new C().m();",
                "number(1)",
            ),
            (
                "let C = class extends null { constructor() { return Object.create(new.target.prototype); } }; new C() instanceof C;",
                "boolean(true)",
            ),
            (
                "class C extends null { x = 1; constructor() { return Object.create(new.target.prototype); } } new C().x;",
                "undefined(undefined)",
            ),
            (
                "try { class C extends null {} new C(); } catch (e) { e.name; }",
                "string(TypeError)",
            ),
            (
                "try { class C extends null { constructor() { super(); } } new C(); } catch (e) { e.name; }",
                "string(TypeError)",
            ),
            (
                "try { class C extends null { constructor() { return undefined; } } new C(); } catch (e) { e.name; }",
                "string(ReferenceError)",
            ),
            (
                "try { class C extends null { m() { return super.x; } constructor() { return Object.create(new.target.prototype); } } new C().m(); } catch (e) { e.name; }",
                "string(TypeError)",
            ),
            (
                "try { class C extends null { #x = 1; read() { return this.#x; } constructor() { return Object.create(new.target.prototype); } } new C().read(); } catch (e) { e.name; }",
                "string(TypeError)",
            ),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| panic!("phase 30 case should run for `{source}`: {err:?}"));
            assert!(outcome.note.contains(expected), "source: {source}, note: {}", outcome.note);
        }
    }

    #[test]
    fn wasm_backend_supports_phase_twenty_six_builtin_globals() {
        for (source, expected) in [
            ("typeof Function;", "string(function)"),
            ("Function === globalThis.Function;", "boolean(true)"),
            ("function f() {} f instanceof Function;", "boolean(true)"),
            ("class C {} C instanceof Function;", "boolean(true)"),
            (
                "Object.getPrototypeOf(function f(){}) === Function.prototype;",
                "boolean(true)",
            ),
            ("Error(\"x\").message;", "string(x)"),
            ("new Error(\"x\") instanceof Error;", "boolean(true)"),
            ("RangeError(\"x\").name;", "string(RangeError)"),
            ("new RangeError(\"x\") instanceof Error;", "boolean(true)"),
            ("new SyntaxError(\"x\") instanceof Error;", "boolean(true)"),
            ("new EvalError(\"x\").name;", "string(EvalError)"),
            ("new URIError(\"x\").message;", "string(x)"),
            ("new TypeError(\"x\") instanceof Error;", "boolean(true)"),
            ("new ReferenceError(\"x\").name;", "string(ReferenceError)"),
            (
                "try { \"x\" in 1; } catch (e) { e instanceof TypeError; }",
                "boolean(true)",
            ),
            (
                "try { class C {} C(); } catch (e) { e instanceof TypeError; }",
                "boolean(true)",
            ),
            ("Object.create({ x: 1 }).x;", "number(1)"),
            (
                "let p = { x: 1 }; let o = Object.create(p); Object.getPrototypeOf(o) === p;",
                "boolean(true)",
            ),
            ("({}) instanceof Object;", "boolean(true)"),
            ("[] instanceof Array;", "boolean(true)"),
            ("Array.isArray([]);", "boolean(true)"),
            ("Array.isArray({});", "boolean(false)"),
            ("Array(1, 2)[1];", "number(2)"),
            ("new Array(\"x\")[0];", "string(x)"),
            ("let o = {}; Object(o) === o;", "boolean(true)"),
            (
                "function add(x, y) { return x + y; } add.call(null, 1, 2);",
                "number(3)",
            ),
            (
                "function f(x) { return this.v + x; } let o = { v: 2 }; f.call(o, 3);",
                "number(5)",
            ),
            (
                "function add(x, y) { return x + y; } add.apply(null, [1, 2]);",
                "number(3)",
            ),
            (
                "function pick() { return arguments[1]; } pick.apply(null, [1, 2, 3]);",
                "number(2)",
            ),
            (
                "function f() { return this.x; } let o = { x: 4 }; f.apply(o, []);",
                "number(4)",
            ),
            (
                "AggregateError([1, 2], \"x\").name;",
                "string(AggregateError)",
            ),
            (
                "new AggregateError([1, 2], \"x\") instanceof Error;",
                "boolean(true)",
            ),
            (
                "new AggregateError([1, 2], \"x\") instanceof AggregateError;",
                "boolean(true)",
            ),
            (
                "let e = AggregateError([1, undefined, 3], \"x\"); e.errors[1];",
                "undefined(undefined)",
            ),
            (
                "let cause = { marker: 1 }; new Error(\"m\", { cause: cause }).cause === cause;",
                "boolean(true)",
            ),
            (
                "let cause = { marker: 1 }; new AggregateError([], \"m\", { cause: cause }).cause === cause;",
                "boolean(true)",
            ),
            (
                "Object.prototype.hasOwnProperty.call(new AggregateError([], \"m\"), \"cause\");",
                "boolean(false)",
            ),
            ("AggregateError.length;", "number(2)"),
            (
                "Object.getPrototypeOf(AggregateError) === Error;",
                "boolean(true)",
            ),
            (
                "AggregateError.prototype.constructor === AggregateError;",
                "boolean(true)",
            ),
            (
                "AggregateError === globalThis.AggregateError;",
                "boolean(true)",
            ),
            ("AggregateError instanceof Function;", "boolean(true)"),
            (
                "class C {} try { C.call({}); } catch (e) { e instanceof TypeError; }",
                "boolean(true)",
            ),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| {
                    panic!("phase 26 builtin case should run for `{source}`: {err:?}")
                });
            assert!(
                outcome.note.contains(expected),
                "source: {source}, note: {}",
                outcome.note
            );
        }
    }

    #[test]
    fn wasm_backend_propagates_aggregate_error_iterator_getter_throw() {
        let source = "function E(message) { this.message = message; } let it = { get [Symbol.iterator]() { throw new E(\"boom\"); } }; try { new AggregateError(it); } catch (e) { e instanceof E && e.constructor === E && e.message === \"boom\"; }";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("AggregateError iterator getter throw should run: {err:?}")
            });
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_emits_arraybuffer_bytelength_getter_for_property_access() {
        let outcome = engine()
            .run_script(
                "new ArrayBuffer(42).byteLength;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("ArrayBuffer byteLength property access should run: {err:?}")
            });
        assert!(
            outcome.note.contains("number(42)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_emits_arraybuffer_resize_for_closure_method_call() {
        let source = "const rab = new ArrayBuffer(64, { maxByteLength: 1024 }); let called = false; let threw = false; try { (() => rab.resize({ valueOf() { called = true; throw new TypeError(); }}))(); } catch (e) { threw = e instanceof TypeError; } called + '|' + threw;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("ArrayBuffer resize method call in closure should run: {err:?}")
            });
        assert!(
            outcome.note.contains("string(true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_writes_captured_typed_array_indexes() {
        let source = "const view = new Uint8Array(new ArrayBuffer(4)); function write() { view[1] = 9; } write(); view[1];";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("captured typed array write should run: {err:?}"));
        assert!(outcome.note.contains("number(9)"), "note: {}", outcome.note);
    }

    #[test]
    fn wasm_backend_validates_slice_named_function_property_reads() {
        let source = "let o = { slice() { return 1; } }; typeof o.slice === 'function' && ArrayBuffer.prototype.slice.length === 2;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("slice-named function property reads should validate and run: {err:?}")
            });
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_map_result_indexes_are_ordinary_data_properties() {
        let source = "var r = [1, 2, 3].map(function(v, i) { return i + 10; }); var before = Object.keys(r).join(','); var d = Object.getOwnPropertyDescriptor(r, 1); r[1] = 42; delete r[2]; before + '|' + r[1] + '|' + r[2] + '|' + d.writable + '|' + d.enumerable + '|' + d.configurable + '|' + Object.keys(r).join(',');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("Array.prototype.map result descriptors should run: {err:?}")
            });
        assert!(
            outcome
                .note
                .contains("string(0,1,2|42|undefined|true|true|true|0,1)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_map_remains_generic_for_inherited_array_methods() {
        let source = "function Receiver() {} Receiver.prototype = new Array(1, 2, 3); var receiver = new Receiver(); receiver.length = 1; var calls = 0; var result = receiver.map(function(value, index, array) { calls += 1; return array === receiver && index === 0 && value === 1; }); Array.isArray(result) + '|' + result.length + '|' + result[0] + '|' + calls;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.map should remain generic on non-array receivers");
        assert!(
            outcome.note.contains("string(true|1|true|1)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_callback_methods_remain_generic_when_inherited() {
        let source = "function Receiver() {} Receiver.prototype = new Array(1, 2, 3); var receiver = new Receiver(); receiver.length = 1; receiver.every(function(value) { return value === 1; }) + '|' + receiver.some(function(value) { return value === 1; }) + '|' + receiver.filter(function(value) { return value === 1; }).join(',');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("inherited Array callback methods should remain generic");
        assert!(
            outcome.note.contains("string(true|true|1)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_methods_use_observable_arguments_length() {
        let source = "function check(a, b) { arguments[2] = 9; var flat = Array.prototype.flat.call(arguments); var flatMapped = Array.prototype.flatMap.call(arguments, function(value) { return [value]; }); var mapped = Array.prototype.map.call(arguments, function(value) { return value; }); var filtered = Array.prototype.filter.call(arguments, function() { return true; }); var every = Array.prototype.every.call(arguments, function(value) { return value > 10; }); var some = Array.prototype.some.call(arguments, function(value) { return value === 9; }); var index = Array.prototype.indexOf.call(arguments, 9); var lastIndex = Array.prototype.lastIndexOf.call(arguments, 9); return flat.length + '|' + flatMapped.length + '|' + mapped.length + '|' + filtered.length + '|' + every + '|' + some + '|' + index + '|' + lastIndex; } check(12, 11);";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array methods should use the observable arguments length");
        assert!(
            outcome.note.contains("string(2|2|2|2|true|false|-1|-1)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_search_methods_stop_after_index_getter_throws() {
        let source = "var forwardAccessed = false; var forward = []; Object.defineProperty(forward, '0', { get: function() { throw new TypeError(); } }); Object.defineProperty(forward, '1', { get: function() { forwardAccessed = true; return true; } }); var forwardCaught = false; try { forward.indexOf(true); } catch (error) { forwardCaught = error instanceof TypeError; } var reverseAccessed = false; var reverse = []; Object.defineProperty(reverse, '0', { get: function() { reverseAccessed = true; return true; } }); Object.defineProperty(reverse, '1', { get: function() { throw new TypeError(); } }); var reverseCaught = false; try { reverse.lastIndexOf(true); } catch (error) { reverseCaught = error instanceof TypeError; } forwardCaught + '|' + forwardAccessed + '|' + reverseCaught + '|' + reverseAccessed;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array search methods should propagate indexed getter exceptions");
        assert!(
            outcome.note.contains("string(true|false|true|false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_concat_stops_after_index_getter_throws() {
        let source = "var token = {}; var accessed = false; var values = []; Object.defineProperty(values, '0', { get: function() { throw token; } }); Object.defineProperty(values, '1', { get: function() { accessed = true; return true; } }); var caught = false; try { values.concat([]); } catch (error) { caught = error === token; } caught + '|' + accessed;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.concat should propagate indexed getter exceptions");
        assert!(
            outcome.note.contains("string(true|false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_to_string_requires_an_object_coercible_receiver() {
        let source = "var arrayMethod = Array.prototype.toString; var typedArrayMethod = Uint8Array.prototype.toString; var arrayThrows = false; var typedArrayThrows = false; try { arrayMethod(); } catch (error) { arrayThrows = error instanceof TypeError; } try { typedArrayMethod(); } catch (error) { typedArrayThrows = error instanceof TypeError; } arrayThrows + '|' + typedArrayThrows;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array toString methods should reject nullish receivers");
        assert!(
            outcome.note.contains("string(true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_to_locale_string_preserves_primitive_receivers() {
        let source = r#"
"use strict";
Boolean.prototype.toString = function() {
  return typeof this;
};
let direct = [true, false].toLocaleString();
Object.defineProperty(Boolean.prototype, "toString", {
  get: function() {
    let receiverType = typeof this;
    return function() {
      return receiverType;
    };
  }
});
direct + "|" + [true, false].toLocaleString();
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.toLocaleString should preserve primitive receivers");
        assert!(
            outcome
                .note
                .contains("string(boolean,boolean|boolean,boolean)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_to_locale_string_snapshots_resizable_typed_array_length() {
        let source = r#"
let original = Number.prototype.toLocaleString;
let shrinkingBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
let shrinking = new Uint8Array(shrinkingBuffer, 0, 4);
let shrinkCalls = 0;
Number.prototype.toLocaleString = function() {
  shrinkCalls += 1;
  if (shrinkCalls === 2) shrinkingBuffer.resize(2);
  return original.call(this);
};
let shrunk = Array.prototype.toLocaleString.call(shrinking);

let growingBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
let growing = new Uint8Array(growingBuffer);
let growCalls = 0;
Number.prototype.toLocaleString = function() {
  growCalls += 1;
  if (growCalls === 2) growingBuffer.resize(6);
  return original.call(this);
};
let grown = Array.prototype.toLocaleString.call(growing);
Number.prototype.toLocaleString = original;
shrunk + "|" + grown + "|" + shrinkCalls + "|" + growCalls;
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.toLocaleString should snapshot typed array length");
        assert!(
            outcome.note.contains("string(0,0,,|0,0,0,0|2|4)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_to_locale_string_uses_internal_receiver_state() {
        let source = "const method = Uint8Array.prototype.toLocaleString; let forged = 'missing'; try { forged = method.call({ $TypedArrayViewedArrayBuffer: {}, length: 1, 0: 1 }); } catch (error) { forged = error.name; } const values = new Uint8Array([1, 2]); let lengthReads = 0; Object.defineProperty(values, 'length', { get() { lengthReads++; return 0; } }); const typed = method.call(values); const generic = Array.prototype.toLocaleString.call({ length: 2, 0: 1, 1: 2 }); forged + '|' + typed + '|' + lengthReads + '|' + generic;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray.prototype.toLocaleString should use internal receiver state");
        assert!(
            outcome.note.contains("string(TypeError|1,2|0|1,2)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_to_locale_string_calls_proxies_with_a_length_snapshot() {
        let source = "const original = Number.prototype.toLocaleString; const buffer = new ArrayBuffer(4, { maxByteLength: 8 }); const values = new Uint8Array(buffer); values[0] = 1; values[1] = 2; values[2] = 3; values[3] = 4; const receivers = []; function format() { 'use strict'; receivers.push(typeof this); if (receivers.length === 1) buffer.resize(2); return Number(this) * 10; } Number.prototype.toLocaleString = new Proxy(format, {}); const result = values.toLocaleString(); Number.prototype.toLocaleString = original; result + '|' + receivers.length + '|' + receivers.join(',');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray.prototype.toLocaleString should call callable proxies");
        assert!(
            outcome.note.contains("string(10,20,,|2|number,number)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_from_snapshots_proxy_iterators_before_mapping() {
        let source = "let nextCalls = 0; const iterator = { next: new Proxy(function() { nextCalls++; return nextCalls <= 3 ? { value: nextCalls } : { done: true }; }, {}) }; const source = { [Symbol.iterator]: new Proxy(function() { return iterator; }, {}) }; const receiver = { factor: 2 }; const mapper = new Proxy(function(value, index) { return value * this.factor + index; }, {}); const Constructor = new Proxy(function(length) { return new Uint8Array(length); }, {}); const result = Uint8Array.from.call(Constructor, source, mapper, receiver); result.join(',') + '|' + result.length + '|' + nextCalls;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray.from should consume proxy iterators before mapping");
        assert!(
            outcome.note.contains("string(2,5,8|3|4)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_from_and_of_use_generic_constructor_semantics() {
        let source = "const events = []; const source = { get [Symbol.iterator]() { events.push('iterator'); return null; }, get length() { events.push('length'); return 2.9; }, get 0() { events.push('get:0'); return 4; }, get 1() { events.push('get:1'); return 5; } }; const Constructor = new Proxy(function(length) { events.push('construct:' + length); return new Uint8Array(length); }, {}); const mapper = new Proxy(function(value, index) { events.push('map:' + index); return value + index; }, {}); const from = Uint8Array.from.call(Constructor, source, mapper); const of = Uint8Array.of.call(Constructor, 1, 258); let forged = 'missing'; try { Uint8Array.of.call(function() { return { $TypedArrayViewedArrayBuffer: {}, $TypedArrayByteOffset: 0, $TypedArrayByteLength: 2, $TypedArrayBytesPerElement: 1, $TypedArrayElementKind: 1 }; }, 1, 2); } catch (error) { forged = error.name; } from.join(',') + '|' + of.join(',') + '|' + forged + '|' + events.join(',');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray.from and TypedArray.of should use generic constructors");
        assert!(
            outcome.note.contains("string(4,6|1,2|TypeError|iterator,length,construct:2,get:0,map:0,get:1,map:1,construct:2)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_accessors_use_internal_resizable_view_state() {
        let source = "const prototype = Object.getPrototypeOf(Uint8Array.prototype); const names = ['buffer', 'byteLength', 'byteOffset', 'length']; const getters = names.map(name => Object.getOwnPropertyDescriptor(prototype, name).get); const rab = new ArrayBuffer(8, { maxByteLength: 16 }); const fixed = new Uint16Array(rab, 2, 2); const tracking = new Uint16Array(rab, 2); fixed.$TypedArrayByteLength = 100; fixed.$TypedArrayByteOffset = 100; function state(value) { return (getters[0].call(value) === rab) + ':' + getters[1].call(value) + ':' + getters[2].call(value) + ':' + getters[3].call(value); } const initial = state(fixed) + '|' + state(tracking); rab.resize(3); const shrunk = state(fixed) + '|' + state(tracking); rab.resize(8); const restored = state(fixed) + '|' + state(tracking); const forged = { $TypedArrayViewedArrayBuffer: rab, $TypedArrayByteLength: 4, $TypedArrayByteOffset: 0, $TypedArrayBytesPerElement: 1 }; let forgedErrors = 0; let proxyErrors = 0; for (const getter of getters) { try { getter.call(forged); } catch (error) { if (error.name === 'TypeError') forgedErrors++; } try { getter.call(new Proxy(fixed, {})); } catch (error) { if (error.name === 'TypeError') proxyErrors++; } } initial + '|' + shrunk + '|' + restored + '|' + forgedErrors + '|' + proxyErrors;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray accessors should use internal resizable view state");
        assert!(
            outcome.note.contains(
                "string(true:4:2:2|true:6:2:3|true:0:0:0|true:0:2:0|true:4:2:2|true:6:2:3|4|4)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_for_each_observes_inherited_indexes_after_deletion() {
        let source = "Array.prototype[4] = 9; var array = [1, 2, 3, 4, 5]; var values = []; array.forEach(function(value, index) { if (index === 0) delete array[4]; values.push(index + ':' + value); }); delete Array.prototype[4]; values.join(',');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.forEach should observe inherited indexes after deletion");
        assert!(
            outcome.note.contains("string(0:1,1:2,2:3,3:4,4:9)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_compound_assigns_constant_array_indexes() {
        let source = "var array = [10]; var result = (array[0] += 2); var strings = ['a']; strings[0] += 2; var bigints = [10n]; bigints[0] += 2n; result + '|' + array[0] + '|' + strings[0] + '|' + bigints[0];";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("compound assignment should support constant computed array indexes");
        assert!(
            outcome.note.contains("string(12|12|a2|12)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_accessor_redefinition_preserves_omitted_flags() {
        let source = "var array = [0, 1, 2]; Object.defineProperty(array, '1', { get: function() { return 1; } }); var descriptor = Object.getOwnPropertyDescriptor(array, '1'); array.length = 1; descriptor.enumerable + '|' + descriptor.configurable + '|' + array.length;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("array accessor redefinition should preserve existing index flags");
        assert!(
            outcome.note.contains("string(true|true|1)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_to_reversed_snapshots_length_and_reads_in_descending_order() {
        let source = "var order = []; var array = [0, 1, 2, 3, 4]; Array.prototype[1] = 5; Object.defineProperty(array, '3', { get: function() { order.push(3); array.length = 1; return 3; } }); var result = array.toReversed(); delete Array.prototype[1]; result.join(',') + '|' + order.join(',');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.toReversed should use its length snapshot");
        assert!(
            outcome.note.contains("string(4,3,,5,0|3)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_with_replaces_without_reading_the_selected_index() {
        let source = "var reads = []; var source = { length: 3, get 0() { reads.push(0); return 1; }, get 1() { throw 'selected index was read'; }, get 2() { reads.push(2); return 3; } }; var result = Array.prototype.with.call(source, -2, 9); result.join(',') + '|' + reads.join(',');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.with should replace the selected index without reading it");
        assert!(
            outcome.note.contains("string(1,9,3|0,2)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_to_spliced_skips_deleted_elements() {
        let source = "var reads = []; var source = { length: 4, get 0() { reads.push(0); return 1; }, get 1() { throw 'deleted element was read'; }, get 2() { throw 'deleted element was read'; }, get 3() { reads.push(3); return 4; } }; var result = Array.prototype.toSpliced.call(source, 1, 2, 8, 9); result.join(',') + '|' + reads.join(',');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.toSpliced should not read deleted elements");
        assert!(
            outcome.note.contains("string(1,8,9,4|0,3)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_to_sorted_reads_all_elements_before_comparing() {
        let source = "var reads = []; var comparedAfterReads = true; var source = { length: 3, get 0() { reads.push(0); return 3; }, get 1() { reads.push(1); return 1; }, get 2() { reads.push(2); return 2; } }; var result = Array.prototype.toSorted.call(source, function(a, b) { if (reads.length !== 3) comparedAfterReads = false; return a - b; }); result.join(',') + '|' + reads.join(',') + '|' + comparedAfterReads;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.toSorted should collect values before comparing");
        assert!(
            outcome.note.contains("string(1,2,3|0,1,2|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_reverse_handles_resizable_typed_array_views() {
        let source = "const buffer = new ArrayBuffer(4, { maxByteLength: 8 }); const fixed = new Uint8Array(buffer, 0, 4); const tracking = new Uint8Array(buffer); fixed[0] = 1; fixed[1] = 2; fixed[2] = 3; fixed[3] = 4; Array.prototype.reverse.call(fixed); const first = tracking.join(','); buffer.resize(3); tracking[0] = 5; tracking[1] = 6; tracking[2] = 7; Array.prototype.reverse.call(fixed); Array.prototype.reverse.call(tracking); first + '|' + tracking.join(',');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.reverse should handle resizable typed array views");
        assert!(
            outcome.note.contains("string(4,3,2,1|7,6,5)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_join_validates_receiver_and_uses_current_view() {
        let source = "const buffer = new ArrayBuffer(4, { maxByteLength: 8 }); const values = new Uint8Array(buffer); values[0] = 1; values[1] = 2; values[2] = 3; values[3] = 4; const before = values.join('-'); buffer.resize(2); const after = values.join(','); let lengthReads = 0; Object.defineProperty(values, 'length', { get() { lengthReads++; return 0; } }); const internalLength = values.join(':'); let borrowed = 'missing'; try { Uint8Array.prototype.join.call({ length: 1, 0: 1 }); } catch (error) { borrowed = error.name; } before + '|' + after + '|' + internalLength + '|' + lengthReads + '|' + borrowed + '|' + (values.join === Array.prototype.join);";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray.prototype.join should use TypedArray semantics");
        assert!(
            outcome
                .note
                .contains("string(1-2-3-4|1,2|1:2|0|TypeError|false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_join_validation_does_not_change_generic_array_join() {
        let source = "const buffer = new ArrayBuffer(4, { maxByteLength: 8 }); const fixed = new Uint8Array(buffer, 1, 2); buffer.resize(2); let typedResult = 'missing'; try { Uint8Array.prototype.join.call(fixed); } catch (error) { typedResult = error.name; } const genericResult = Array.prototype.join.call(fixed); let objectResult = 'missing'; try { Uint8Array.prototype.join.call({ length: 1, 0: 1 }); } catch (error) { objectResult = error.name; } const genericObjectResult = Array.prototype.join.call({ length: 1, 0: 1 }); typedResult + '|' + genericResult + '|' + objectResult + '|' + genericObjectResult;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray join validation should not change generic Array join");
        assert!(
            outcome.note.contains("string(TypeError||TypeError|1)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_subarray_uses_species_and_preserves_the_backing_buffer() {
        let source = "const buffer = new ArrayBuffer(8); const source = new Uint16Array(buffer); source[0] = 1; source[1] = 2; source[2] = 3; source[3] = 4; let observed = ''; source.constructor = { [Symbol.species]: function(speciesBuffer, byteOffset, length) { observed = (speciesBuffer === buffer) + ':' + byteOffset + ':' + length; return new Uint8Array(speciesBuffer, byteOffset, length); } }; const result = source.subarray(1, 3); const replacement = new Uint8Array([9, 8, 7]); source.constructor = { [Symbol.species]: function() { return replacement; } }; const preserved = source.subarray(0, 0); observed + '|' + (result.buffer === buffer) + ':' + result.byteOffset + ':' + result.length + ':' + result[0] + '|' + (preserved === replacement);";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray.prototype.subarray should use the selected species constructor");
        assert!(
            outcome.note.contains("string(true:2:2|true:2:2:2|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_subarray_observes_resize_and_detach_boundaries() {
        let source = "const buffer = new ArrayBuffer(4, { maxByteLength: 8 }); const tracking = new Uint8Array(buffer); tracking[0] = 1; tracking[1] = 2; tracking[2] = 3; tracking[3] = 4; const result = tracking.subarray(1); buffer.resize(6); const tracked = result.byteOffset + ':' + result.length; const detached = new Uint8Array([1, 2]); let conversions = ''; const begin = { valueOf() { conversions += 'b'; return 0; } }; const end = { valueOf() { conversions += 'e'; return 1; } }; __porfDetachArrayBuffer(detached.buffer); let errorName = 'missing'; try { detached.subarray(begin, end); } catch (error) { errorName = error.name; } tracked + '|' + conversions + ':' + errorName;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray.prototype.subarray should preserve resize and detach ordering");
        assert!(
            outcome.note.contains("string(1:5|be:TypeError)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_at_validates_out_of_bounds_dynamic_receivers() {
        let source = "function readFirst(view) { return view.at(0); } const typedArrayAt = Object.getPrototypeOf(Uint8Array).prototype.at; const buffer = new ArrayBuffer(4, { maxByteLength: 8 }); const fixed = new Uint8Array(buffer, 1, 2); buffer.resize(2); let methodResult = 'missing'; try { readFirst(fixed); } catch (error) { methodResult = error.name; } let intrinsicResult = 'missing'; try { typedArrayAt.call(fixed, 0); } catch (error) { intrinsicResult = error.name; } let objectResult = 'missing'; try { typedArrayAt.call({ length: 1, 0: 1 }, 0); } catch (error) { objectResult = error.name; } let arrayResult = 'missing'; try { typedArrayAt.call([1], 0); } catch (error) { arrayResult = error.name; } const genericResult = Array.prototype.at.call(fixed, 0); methodResult + '|' + intrinsicResult + '|' + objectResult + '|' + arrayResult + '|' + (genericResult === undefined) + '|' + (Uint8Array.prototype.at === Array.prototype.at);";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray.prototype.at should validate out-of-bounds dynamic receivers");
        assert!(
            outcome
                .note
                .contains("string(TypeError|TypeError|TypeError|TypeError|true|false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_join_formats_bigint_elements() {
        let source = "const signed = new BigInt64Array([1n, 0n, 2n, -3n]); const unsigned = new BigUint64Array([1n, 42n]); signed.join(',') + '|' + signed.join(null) + '|' + unsigned.join('-');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray.prototype.join should format BigInt elements");
        assert!(
            outcome
                .note
                .contains("string(1,0,2,-3|1null0null2null-3|1-42)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_typed_array_fill_accepts_number_and_bigint_elements() {
        let source = "const numbers = new Int32Array([1, 2, 3]); const bigints = new BigInt64Array([1n, 2n]); numbers.fill(7, 1); bigints.fill(-5n); numbers.join(',') + '|' + bigints.join(',');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray.prototype.fill should write Number and BigInt elements");
        assert!(
            outcome.note.contains("string(1,7,7|-5,-5)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_integer_typed_array_writes_apply_modulo_conversions() {
        let source = r#"
const u32 = new Uint32Array(6);
u32[0] = 4294967295;
u32[1] = 4294967296;
u32[2] = -1;
u32[3] = -4294967295;
u32[4] = Infinity;
u32[5] = NaN;
const i32 = new Int32Array(4);
i32[0] = 4294967295;
i32[1] = 2147483648;
i32[2] = -2147483649;
i32[3] = 4294967297.9;
const u16 = new Uint16Array(4);
u16[0] = 65535;
u16[1] = 65536;
u16[2] = -1;
u16[3] = -65535;
const i16 = new Int16Array(3);
i16[0] = 65535;
i16[1] = 32768;
i16[2] = -32769;
const u8 = new Uint8Array(4);
u8[0] = 255;
u8[1] = 256;
u8[2] = -1;
u8[3] = -255;
const i8 = new Int8Array(3);
i8[0] = 255;
i8[1] = 128;
i8[2] = -129;
const clamped = new Uint8ClampedArray(4);
clamped[0] = 300;
clamped[1] = -1;
clamped[2] = 0.5;
clamped[3] = 1.5;
u32.join(",") + "|" + i32.join(",") + "|" +
  u16.join(",") + "|" + i16.join(",") + "|" +
  u8.join(",") + "|" + i8.join(",") + "|" + clamped.join(",");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("integer typed array writes should apply modulo conversions");
        assert!(
            outcome.note.contains(
                "string(4294967295,0,4294967295,1,0,0|-1,-2147483648,2147483647,1|65535,0,65535,1|-1,-32768,32767|255,0,255,1|-1,-128,127|255,0,0,2)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_define_property_writes_typed_array_elements() {
        let source = r#"
const values = new Uint32Array([0]);
const objectResult = Object.defineProperty(values, "0", { value: 4294967295 });
const reflectResult = Reflect.defineProperty(values, "0", { value: 4294967296 });
let incompatibleThrew = false;
let incompatibleConverted = false;
try {
  Object.defineProperty(values, "0", {
    value: { valueOf() { incompatibleConverted = true; return 1; } },
    writable: false
  });
} catch (error) {
  incompatibleThrew = error instanceof TypeError;
}
let conversionError = "none";
try {
  Object.defineProperty(values, "0", {
    value: { valueOf() { throw new RangeError("conversion"); } }
  });
} catch (error) {
  conversionError = error.name;
}
(objectResult === values) + "|" + reflectResult + "|" + values[0] + "|" +
  incompatibleThrew + "|" + incompatibleConverted + "|" + conversionError;
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("defineProperty should use TypedArray integer-indexed semantics");
        assert!(
            outcome
                .note
                .contains("string(true|true|0|true|false|RangeError)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_dataview_get_biguint64_returns_unsigned_bigints() {
        let source = r#"
const buffer = new ArrayBuffer(8);
const view = new DataView(buffer);
for (let index = 0; index < 8; index++) view.setUint8(index, 255);
const maximum = view.getBigUint64(0);
const reversed = view.getBigUint64(0, true);
(maximum === 18446744073709551615n) + "|" +
  (reversed === 18446744073709551615n) + "|" +
  (maximum === 9223372036854775807n) + "|" +
  (18446744073709551615n === 18446744073709551615n) + "|" +
  (typeof maximum) + "|" + (!!maximum);
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("getBigUint64 should preserve the full unsigned 64-bit range");
        assert!(
            outcome
                .note
                .contains("string(true|true|false|true|bigint|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_compares_multi_limb_bigint_literals_by_value() {
        let source = r#"
const positive = 340282366920938463463374607431768211456n;
const negative = -340282366920938463463374607431768211456n;
(positive === 340282366920938463463374607431768211456n) + "|" +
  (positive !== 340282366920938463463374607431768211457n) + "|" +
  (negative === -340282366920938463463374607431768211456n) + "|" +
  (positive === negative) + "|" +
  (positive == 340282366920938463463374607431768211456n) + "|" +
  (positive != 340282366920938463463374607431768211457n) + "|" +
  (typeof positive) + "|" + (!!positive);
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("multi-limb BigInt literals should preserve mathematical identity");
        assert!(
            outcome
                .note
                .contains("string(true|true|true|false|true|true|bigint|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_atomics_add_returns_old_values_and_updates_shared_integer_views() {
        let source = r#"
const i8 = new Int8Array(new SharedArrayBuffer(1));
i8[0] = -5;
const oldI8 = Atomics.add(i8, 0, 10);
const u8 = new Uint8Array(new SharedArrayBuffer(1));
u8[0] = 250;
const oldU8 = Atomics.add(u8, 0, 10);
const i16 = new Int16Array(new SharedArrayBuffer(2));
i16[0] = -32000;
const oldI16 = Atomics.add(i16, 0, 1000);
const u16 = new Uint16Array(new SharedArrayBuffer(2));
u16[0] = 65000;
const oldU16 = Atomics.add(u16, 0, 1000);
const i32 = new Int32Array(new SharedArrayBuffer(4));
i32[0] = 2147483647;
const oldI32 = Atomics.add(i32, 0, 1);
const u32 = new Uint32Array(new SharedArrayBuffer(4));
u32[0] = 123456789;
const oldU32 = Atomics.add(u32, 0, 2);
const i64 = new BigInt64Array(new SharedArrayBuffer(8));
i64[0] = -5n;
const oldI64 = Atomics.add(i64, 0, 12n);
const u64 = new BigUint64Array(new SharedArrayBuffer(8));
u64[0] = 123456789n;
const oldU64 = Atomics.add(u64, 0, 2n);

oldI8 + ":" + i8[0] + "|" +
oldU8 + ":" + u8[0] + "|" +
oldI16 + ":" + i16[0] + "|" +
oldU16 + ":" + u16[0] + "|" +
oldI32 + ":" + i32[0] + "|" +
oldU32 + ":" + u32[0] + "|" +
oldI64 + ":" + i64[0] + "|" +
oldU64 + ":" + u64[0];
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Atomics.add should update shared integer typed arrays");
        assert!(
            outcome.note.contains(
                "string(-5:5|250:4|-32000:-31000|65000:464|2147483647:-2147483648|123456789:123456791|-5:7|123456789:123456791)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_atomics_add_rejects_out_of_bounds_indices() {
        let source = r#"
const ints = new Int32Array(new SharedArrayBuffer(8));
const bigs = new BigInt64Array(new SharedArrayBuffer(16));
const errors = [];
try {
  Atomics.add(ints, -1, 1);
} catch (error) {
  errors.push(error.name);
}
try {
  Atomics.add(bigs, bigs.length, 1n);
} catch (error) {
  errors.push(error.name);
}
errors.join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Atomics.add should reject out-of-bounds indices");
        assert!(
            outcome.note.contains("string(RangeError|RangeError)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_atomics_load_and_store_preserve_integer_values_and_validation_order() {
        let source = r#"
const i8 = new Int8Array(new ArrayBuffer(1));
i8[0] = -1;
const u32 = new Uint32Array(new SharedArrayBuffer(4));
u32[0] = 4294967295;
const i64 = new BigInt64Array(new SharedArrayBuffer(8));
i64[0] = -7n;

const stored = Atomics.store(u32, 0, Math.PI);
const objectStored = Atomics.store(u32, 0, { valueOf() { return 33; } });
const negativeStored = Atomics.store(i8, 0, -5);
const negativeZero = Atomics.store(i8, 0, -0);

let indexCoerced = false;
const poisonedIndex = { valueOf() { indexCoerced = true; return 0; } };
let loadTypeError = false;
try {
  Atomics.load(new Float32Array(new SharedArrayBuffer(8)), poisonedIndex);
} catch (error) {
  loadTypeError = error instanceof TypeError;
}

let valueCoerced = false;
const poisonedValue = { valueOf() { valueCoerced = true; return 1; } };
let storeTypeError = false;
try {
  Atomics.store(new Uint8ClampedArray(new SharedArrayBuffer(8)), 0, poisonedValue);
} catch (error) {
  storeTypeError = error instanceof TypeError;
}

Atomics.load(i8, 0) + "|" + Atomics.load(u32, 0) + "|" + Atomics.load(i64, 0) + "|" +
  stored + "|" + objectStored + "|" + negativeStored + "|" + Object.is(negativeZero, 0) + "|" +
  loadTypeError + "|" + indexCoerced + "|" + storeTypeError + "|" + valueCoerced;
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect(
                "Atomics.load and Atomics.store should implement integer typed array semantics",
            );
        assert!(
            outcome
                .note
                .contains("string(0|33|-7|3|33|-5|true|true|false|true|false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_atomics_notify_validates_views_and_returns_zero_without_waiters() {
        let source = r#"
const sharedI32 = new Int32Array(new SharedArrayBuffer(8));
const localI32 = new Int32Array(new ArrayBuffer(8));
const sharedI64 = new BigInt64Array(new SharedArrayBuffer(16));
const localI64 = new BigInt64Array(new ArrayBuffer(16));
const results = [
  Atomics.notify(sharedI32, 0),
  Atomics.notify(localI32, 0, 1),
  Atomics.notify(sharedI64, 0, Infinity),
  Atomics.notify(localI64, 0, -1)
];

let indexCoerced = false;
let countCoerced = false;
const poisonedIndex = { valueOf() { indexCoerced = true; return 0; } };
const poisonedCount = { valueOf() { countCoerced = true; return 0; } };
let wrongViewThrew = false;
try {
  Atomics.notify(new BigUint64Array(new SharedArrayBuffer(8)), poisonedIndex, poisonedCount);
} catch (error) {
  wrongViewThrew = error instanceof TypeError;
}

let countErrorPreserved = false;
const countError = new RangeError("count");
try {
  Atomics.notify(sharedI64, 0, { valueOf() { throw countError; } });
} catch (error) {
  countErrorPreserved = error === countError;
}

results.join("|") + "|" + wrongViewThrew + "|" + indexCoerced + "|" + countCoerced + "|" + countErrorPreserved;
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Atomics.notify should validate supported views without fabricating waiters");
        assert!(
            outcome
                .note
                .contains("string(0|0|0|0|true|false|false|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_slice_uses_current_resizable_typed_array_bounds() {
        let source = "const buffer = new ArrayBuffer(4, { maxByteLength: 8 }); const tracking = new Uint8Array(buffer); tracking[0] = 1; tracking[1] = 2; tracking[2] = 3; tracking[3] = 4; const grow = { valueOf() { buffer.resize(6); return 0; } }; Array.prototype.slice.call(tracking, grow); const grown = Array.prototype.slice.call(tracking, 3, 5); const fixed = new Uint8Array(buffer, 0, 4); const shrink = { valueOf() { buffer.resize(2); return 0; } }; const fixedResult = Array.prototype.slice.call(fixed, shrink); const trackingResult = Array.prototype.slice.call(tracking); grown.join(',') + '|' + fixedResult.length + '|' + fixedResult.hasOwnProperty(0) + '|' + trackingResult.join(',');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.slice should use current resizable typed array bounds");
        assert!(
            outcome.note.contains("string(4,0|4|false|1,2)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_pop_is_generic_and_preserves_inherited_indexes() {
        let source = "const object = { 0: 'a', length: 1 }; const value = Array.prototype.pop.call(object); const empty = {}; const emptyResult = Array.prototype.pop.call(empty); let stringThrows = false; try { Array.prototype.pop.call(''); } catch (error) { stringThrows = error instanceof TypeError; } Array.prototype[1] = 7; const array = [1]; array.length = 2; const inherited = array.pop(); const remains = array[1]; delete Array.prototype[1]; value + '|' + object.length + '|' + !object.hasOwnProperty('0') + '|' + (emptyResult === undefined) + '|' + empty.length + '|' + (Array.prototype.pop.call(true) === undefined) + '|' + stringThrows + '|' + inherited + '|' + remains;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.pop should work on generic receivers");
        assert!(
            outcome
                .note
                .contains("string(a|0|true|true|0|true|true|7|7)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_prototype_exposes_standard_unscopables() {
        let source = "const value = Array.prototype[Symbol.unscopables]; const descriptor = Object.getOwnPropertyDescriptor(Array.prototype, Symbol.unscopables); Object.getPrototypeOf(value) === null && descriptor.value === value && descriptor.writable === false && descriptor.enumerable === false && descriptor.configurable === true && value.copyWithin === true && value.findLast === true && value.toSpliced === true && !Object.prototype.hasOwnProperty.call(value, 'with');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype should expose the standard unscopables object");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_copy_within_preserves_overlap_and_holes() {
        let source = "var values = [0, 1, , 3]; var result = values.copyWithin(1, 0, 3); (result === values) + '|' + values.join(',') + '|' + values.hasOwnProperty(3);";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.copyWithin should preserve overlap and holes");
        assert!(
            outcome.note.contains("string(true|0,0,1,|false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_concat_rejects_result_above_safe_integer_limit() {
        let source = "var source = { length: Number.MAX_SAFE_INTEGER }; source[Symbol.isConcatSpreadable] = true; var threw = false; try { [1].concat(source); } catch (error) { threw = error instanceof TypeError; } threw;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.concat should reject an oversized result");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_map_visits_arguments_indexes() {
        let source = "function callbackfn(val, idx, obj) { return val + ':' + idx + ':' + Object.prototype.toString.call(obj); } var r = (function(a, b) { return Array.prototype.map.call(arguments, callbackfn); })(9, 11); r.length + '|' + r[0] + '|' + r[1] + '|' + Object.keys(r).join(',');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("Array.prototype.map should visit Arguments indexes: {err:?}")
            });
        assert!(
            outcome
                .note
                .contains("string(2|9:0:[object Arguments]|11:1:[object Arguments]|0,1)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_map_rechecks_prototype_holes_after_getter_side_effects() {
        let source = "function callbackfn(val, idx) { return idx === 1 && val === 6.99 ? false : true; } var arr = [0, , 2]; Object.defineProperty(arr, '0', { get: function() { Object.defineProperty(Array.prototype, '1', { get: function() { return 6.99; }, configurable: true }); return 0; }, configurable: true }); var r = arr.map(callbackfn); delete Array.prototype[1]; r[0] + '|' + r[1] + '|' + r[2] + '|' + Object.keys(r).join(',');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("Array.prototype.map should re-check prototype holes: {err:?}")
            });
        assert!(
            outcome.note.contains("string(true|false|true|0,1,2)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_flat_map_uses_real_builtin_and_compact_target_indexes() {
        let source = "var a = [1, 2].flatMap(function(e) { return [e, e * 2]; }); var b = [1, 2].flatMap(function(e) { return e * 2; }); var c = [, 1].flatMap(function(e) { return e; }); function E() {} var threw = false; try { [].flatMap.call({ get length() { throw new E(); } }, function(e) { return e; }); } catch (e) { threw = e instanceof E; } var d = Array.prototype.flatMap.call(true, function() { return 1; }); a.length + '|' + a[0] + '|' + a[1] + '|' + a[2] + '|' + a[3] + '|' + b.length + '|' + b[0] + '|' + b[1] + '|' + c.length + '|' + c[0] + '|' + c[1] + '|' + Object.keys(c).join(',') + '|' + threw + '|' + d.length;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("Array.prototype.flatMap should run through the real Wasm builtin: {err:?}")
            });
        assert!(
            outcome
                .note
                .contains("string(4|1|2|2|4|2|2|4|1|1|undefined|0|true|0)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_concat_boxes_boolean_receiver() {
        let source = "var t = Array.prototype.concat.call(true); var f = Array.prototype.concat.call(false); t.length + '|' + (t[0] instanceof Boolean) + '|' + f.length + '|' + (f[0] instanceof Boolean);";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("Array.prototype.concat should box Boolean receivers: {err:?}")
            });
        assert!(
            outcome.note.contains("string(1|true|1|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_concat_spreads_arguments_without_changing_length() {
        let source = "var args = (function(a, b, c) { return arguments; })(1, 2, 3); args[Symbol.isConcatSpreadable] = true; var r = [].concat(args, args); Object.defineProperty(args, 'length', { value: 6 }); var s = [].concat(args); r.length + '|' + r[0] + '|' + r[1] + '|' + r[2] + '|' + r[3] + '|' + r[4] + '|' + r[5] + '|' + args.length + '|' + s.length + '|' + s[0] + '|' + s[1] + '|' + s[2] + '|' + s[3] + '|' + s[4] + '|' + s[5];";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("Array.prototype.concat should spread Arguments objects: {err:?}")
            });
        assert!(
            outcome
                .note
                .contains("string(6|1|2|3|1|2|3|6|6|1|2|3|undefined|undefined|undefined)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_concat_spreads_string_wrapper_by_utf16_code_unit() {
        let source = "var str = new String('yuck\\uD83D\\uDCA9'); str[Symbol.isConcatSpreadable] = true; var s = [].concat(str); s.length + '|' + s[0] + s[1] + s[2] + s[3] + '|' + (s[4] === '\\uD83D') + '|' + (s[5] === '\\uDCA9');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!(
                    "Array.prototype.concat should spread String wrappers by UTF-16 code unit: {err:?}"
                )
            });
        assert!(
            outcome.note.contains("string(6|yuck|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_concat_copies_inherited_array_indexes_as_own_properties() {
        let source = "Array.prototype[1] = 1; var x = [0]; x.length = 2; var a = x.concat(); Object.prototype[1] = 1; Object.prototype.length = 2; Object.prototype.concat = Array.prototype.concat; var y = { 0: 0 }; var b = y.concat(); a[0] + '|' + a[1] + '|' + a.hasOwnProperty('1') + '|' + (b[0] === y) + '|' + b[1] + '|' + b.hasOwnProperty('1');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("Array.prototype.concat should copy inherited array indexes: {err:?}")
            });
        assert!(
            outcome.note.contains("string(0|1|true|true|1|false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_concat_propagates_is_concat_spreadable_getter_throw() {
        let source = "function E() {} var o = {}; Object.defineProperty(o, Symbol.isConcatSpreadable, { get: function() { throw new E(); } }); var caught = false; try { Array.prototype.concat.call(o); } catch (e) { caught = e instanceof E; } caught;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!(
                    "Array.prototype.concat should propagate spreadability getter throws: {err:?}"
                )
            });
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_concat_propagates_arguments_index_getter_throw() {
        let source = "function E() {} var args = (function(a) { return arguments; })(1, 2, 3); Object.defineProperty(args, 0, { get: function() { throw new E(); } }); args[Symbol.isConcatSpreadable] = true; var caught = false; try { [].concat(args, args); } catch (error) { caught = error instanceof E; } caught;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.concat should propagate arguments index getter throws");
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_concat_spreads_typed_array_from_dynamic_constructor() {
        let source = "function check(type) { var ta = new type(2); for (var i = 0; i < 2; ++i) { ta[i] = i + 1; } ta[Symbol.isConcatSpreadable] = true; var r = [].concat(ta); return r.length === 2 && r[0] === 1 && r[1] === 2 && r.hasOwnProperty('1'); } check(Uint16Array) + '|' + check(Uint32Array) + '|' + check(Float64Array);";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!(
                    "Array.prototype.concat should spread typed arrays built through dynamic constructors: {err:?}"
                )
            });
        assert!(
            outcome.note.contains("string(true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_reads_typed_array_indexes_from_callback_constructor() {
        let source = "var values = []; [Uint8Array, Uint16Array, Uint32Array, Float32Array, Float64Array].forEach(function(type) { var array = new type(1); values.push(array[0]); }); values.join(':');";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("dynamic property reads should preserve TypedArray integer-index semantics");
        assert!(
            outcome.note.contains("string(0:0:0:0:0)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_array_concat_ignores_cross_realm_array_species() {
        let source = "var array = []; var callCount = 0; var OArray = __porfCreateRealm().global.Array; var speciesDesc = { get: function() { callCount += 1; } }; array.constructor = OArray; Object.defineProperty(Array, Symbol.species, speciesDesc); Object.defineProperty(OArray, Symbol.species, speciesDesc); var result = array.concat(); (Object.getPrototypeOf(result) === Array.prototype) + '|' + callCount;";
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!(
                    "Array.prototype.concat should ignore cross-realm intrinsic Array species: {err:?}"
                )
            });
        assert!(
            outcome.note.contains("string(true|0)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_iterates_captured_array_concat_result() {
        let outcome = engine()
            .run_script(
                r#"
const baseValues = [1, 2];
const values = baseValues.concat(3);

function sumValues() {
  let sum = 0;
  for (const value of values) {
    sum += value;
  }
  return sum;
}

sumValues();
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("captured Array.prototype.concat results should remain iterable arrays");
        assert!(outcome.note.contains("number(6)"), "note: {}", outcome.note);
    }

    #[test]
    fn wasm_backend_preserves_captured_custom_concat_result_kind() {
        let outcome = engine()
            .run_script(
                r#"
const baseValues = [1, 2];
baseValues.concat = function () {
  return 41;
};
const value = baseValues.concat(3);

function describeValue() {
  return typeof value + "|" + (value + 1);
}

describeValue();
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("captured custom concat results should retain their runtime kind");
        assert!(
            outcome.note.contains("string(number|42)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_preserves_captured_concat_species_result_kind() {
        let outcome = engine()
            .run_script(
                r#"
const originalConstructor = Array.prototype.constructor;
const source = [1, 2];
source.concat = Array.prototype.concat;

function Species() {
  return {};
}
function CustomArrayConstructor() {}
CustomArrayConstructor[Symbol.species] = Species;
Array.prototype.constructor = CustomArrayConstructor;

const result = source.concat(3);
Array.prototype.constructor = originalConstructor;

function resultIsArray() {
  return Array.isArray(result);
}

resultIsArray();
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("captured custom Array species results should retain their runtime kind");
        assert!(
            outcome.note.contains("boolean(false)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_supports_phase_twenty_seven_boxed_builtins() {
        for (source, expected) in [
            ("typeof Number;", "string(function)"),
            ("Number === globalThis.Number;", "boolean(true)"),
            ("String === globalThis.String;", "boolean(true)"),
            ("Boolean === globalThis.Boolean;", "boolean(true)"),
            ("new Number(1) instanceof Number;", "boolean(true)"),
            ("new String(\"x\") instanceof String;", "boolean(true)"),
            ("new Boolean(false) instanceof Boolean;", "boolean(true)"),
            ("Object(1) instanceof Number;", "boolean(true)"),
            ("new Object(\"x\") instanceof String;", "boolean(true)"),
            (
                "Object.getPrototypeOf(Object(true)) === Boolean.prototype;",
                "boolean(true)",
            ),
            (
                "function f() { return this instanceof Number; } f.call(1);",
                "boolean(true)",
            ),
            (
                "function f() { return this instanceof String; } f.apply(\"x\", []);",
                "boolean(true)",
            ),
            (
                "function f() { return this instanceof Boolean; } f.call(false);",
                "boolean(true)",
            ),
            ("new Number(1) + 1;", "number(2)"),
            ("new String(\"x\") + \"y\";", "string(xy)"),
            ("new Boolean(false) == false;", "boolean(true)"),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| {
                    panic!("phase 27 boxed builtin case should run for `{source}`: {err:?}")
                });
            assert!(
                outcome.note.contains(expected),
                "source: {source}, note: {}",
                outcome.note
            );
        }
    }

    #[test]
    fn wasm_backend_supports_phase_twenty_eight_bind_and_error_tostring() {
        for (source, expected) in [
            (
                "function add(x, y) { return x + y; } let inc = add.bind(null, 1); inc(2);",
                "number(3)",
            ),
            (
                "function f() { return this; } let o = { v: 2 }; let g = f.bind(o, 3); g() === o;",
                "boolean(true)",
            ),
            (
                "function f() { return this; } let o = { v: 2 }; let g = f.bind(o, 3); g.call({ v: 9 }, 4) === o;",
                "boolean(true)",
            ),
            (
                "function outer() { return (() => this.x).bind({ x: 9 }); } let o = { x: 3, f: outer }; let g = o.f(); g();",
                "number(3)",
            ),
            (
                "function F(x) { this.x = x; } let G = F.bind(null, 2); new G().x;",
                "number(2)",
            ),
            (
                "function F() {} let G = F.bind(null); new G() instanceof F;",
                "boolean(true)",
            ),
            (
                "class C {} let B = C.bind(null); try { B(); } catch (e) { e instanceof TypeError; }",
                "boolean(true)",
            ),
            ("Error(\"x\").toString();", "string(Error: x)"),
            ("TypeError(\"x\").toString();", "string(TypeError: x)"),
            ("let e = new Error(); e.toString();", "string(Error)"),
            (
                "Error.prototype.toString.call({ name: \"X\", message: \"y\" });",
                "string(X: y)",
            ),
            (
                "Error.prototype.toString.call({ name: \"\", message: \"y\" });",
                "string(y)",
            ),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| panic!("phase 28 bind/error-string case should run for `{source}`: {err:?}"));
            assert!(outcome.note.contains(expected), "source: {source}, note: {}", outcome.note);
        }
        let outcome = engine()
            .run_script(
                "try { \"x\" in 1; } catch (e) { e.toString(); }",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("phase 28 TypeError toString case should run: {err:?}"));
        assert!(
            outcome.note.contains("string(TypeError:"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_supports_phase_thirty_one_function_prototype_tostring() {
        for (source, expected) in [
            (
                "function f(x) { return x; } f.toString();",
                "string(function f(x) { return x; })",
            ),
            (
                "let f = x => x + 1; f.toString();",
                "string(x => x + 1)",
            ),
            (
                "let o = { m(x) { return x; } }; o.m.toString();",
                "string(m(x) { return x; })",
            ),
            (
                "class C { constructor(x) { this.x = x; } } C.toString();",
                "string(class C { constructor(x) { this.x = x; } })",
            ),
            (
                "class C { m() { return 1; } } new C().m.toString();",
                "string(m() { return 1; })",
            ),
            (
                "let g = function f(x) { return x; }.bind(null, 1); g.toString();",
                "string(function () { [native code] })",
            ),
            (
                "Function.prototype.toString.call(Function.prototype.call);",
                "string(function call() { [native code] })",
            ),
            (
                "print.toString();",
                "string(function print() { [native code] })",
            ),
            (
                "try { Function.prototype.toString.call({}); } catch (e) { e instanceof TypeError; }",
                "boolean(true)",
            ),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| {
                    panic!("phase 31 function toString case should run for `{source}`: {err:?}")
                });
            assert!(
                outcome.note.contains(expected),
                "source: {source}, note: {}",
                outcome.note
            );
        }
    }

    #[test]
    fn wasm_backend_rejects_phase_twenty_eight_remaining_builtin_tails() {
        for source in [
            "Function(\"return 1\");",
            "new Function(\"return 1\");",
            "function f() { return 1; } f.apply(null, { length: 1, 0: 1 });",
        ] {
            let err = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect_err("phase 26 builtin tail should stay unsupported");
            assert!(
                err.message()
                    .contains("unsupported in porffor wasm-aot first slice"),
                "source: {source}, err: {}",
                err.message()
            );
        }
    }

    #[test]
    fn wasm_backend_supports_phase_twenty_nine_identifier_delete_and_globals() {
        for (source, expected) in [
            ("delete 1;", "boolean(true)"),
            ("let x = 1; delete x;", "boolean(false)"),
            ("const x = 1; delete x;", "boolean(false)"),
            ("var x = 1; delete x; x;", "number(1)"),
            ("function f() {} delete f; typeof f;", "string(function)"),
            (
                "missing = 1; delete missing; typeof missing;",
                "string(undefined)",
            ),
            ("globalThis.x = 1; delete x; typeof x;", "string(undefined)"),
            (
                "globalThis.x = 1; delete globalThis.x; typeof x;",
                "string(undefined)",
            ),
            ("delete missingName;", "boolean(true)"),
            (
                "function f() { y = 3; return delete y; } f();",
                "boolean(true)",
            ),
            ("let x = 1; globalThis.x = 2; delete x; x;", "number(1)"),
            ("delete ({ x: 1 }).x;", "boolean(true)"),
            ("let a = [1, 2]; delete a[0]; a.length;", "number(2)"),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| {
                    panic!("phase 29 delete/global case should run for `{source}`: {err:?}")
                });
            assert!(
                outcome.note.contains(expected),
                "source: {source}, note: {}",
                outcome.note
            );
        }
    }

    #[test]
    fn wasm_backend_surfaces_uncaught_phase_twenty_three_throws() {
        for source in [
            "throw 1;",
            "class C {} C();",
            "class A {} class B extends A { constructor() { this.x = 1; super(); } } new B();",
            "class A {} class B extends A { constructor() {} } new B();",
        ] {
            let err = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect_err("uncaught throw should surface as engine error");
            assert!(
                err.message().starts_with("uncaught throw: "),
                "source: {source}, err: {}",
                err.message()
            );
            if source.contains("extends A") {
                assert!(
                    err.message()
                        .starts_with("uncaught throw: ReferenceError: "),
                    "source: {source}, err: {}",
                    err.message()
                );
            }
        }
    }

    #[test]
    fn wasm_backend_rejects_phase_twenty_four_still_unsupported_edges() {
        for source in [
            "let H; if (true) { H = function() {}; } else { H = print; } class C extends H {} new C();",
            "let H; if (true) { H = null; } else { H = Object; } class C extends H {} new C();",
            "new.target;",
            "class C { #x; m(obj) { delete obj.#x; } }",
            "class C extends Object { m() { delete super.x; } }",
            "class C { async m() {} }",
            "class C { *m() {} }",
        ] {
            let err = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect_err("unsupported class edge should stay unsupported");
            let message = err.message();
            assert!(
                message.contains("unsupported in porffor wasm-aot first slice")
                    || message.contains("parse error")
                    || message.contains("TypeError"),
                "source: {source}, err: {message}"
            );
        }
    }

    #[test]
    fn wasm_backend_map_core_preserves_same_value_zero_and_insertion_updates() {
        for (source, expected) in [
            (
                "var map = new Map(); var object = {}; var symbol = Symbol('k'); map.set('x', 1).set(NaN, 2).set(-0, 3).set(object, 4).set(symbol, 5); map.set('x', 6); [map.size, map.get('x'), map.get(NaN), map.get(+0), map.get(object), map.get(symbol), map.has(-0)].join('|');",
                "string(5|6|2|3|4|5|true)",
            ),
            (
                "var map = new Map(); for (var i = 0; i < 9; i += 1) map.set(i, i * 2); map.delete(2); map.delete(7); map.set(2, 99); [map.size, map.get(2), map.has(7), map.delete(7), map.delete(8), map.size].join('|');",
                "string(8|99|false|false|true|7)",
            ),
            (
                "var map = new Map(); map.set(1, 1).set(2, 2); map.clear(); map.set(3, 3); [map.size, map.has(1), map.get(3)].join('|');",
                "string(1|false|3)",
            ),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| panic!("Map core case should run for `{source}`: {err:?}"));
            assert!(
                outcome.note.contains(expected),
                "source: {source}, note: {}",
                outcome.note
            );
        }
    }

    #[test]
    fn wasm_backend_map_get_or_insert_methods_preserve_keys_and_callback_contract() {
        let outcome = engine()
            .run_script(
                r#"
var map = new Map();
var objectKey = {};
var symbolKey = Symbol("key");
var directMiss = map.getOrInsert(objectKey, "object");
var directHit = map.getOrInsert(objectKey, "replacement");
map.set(NaN, "nan");
var nanHit = map.getOrInsert(NaN, "replacement");
var zeroMap = new Map();
var zeroMiss = zeroMap.getOrInsert(-0, "zero");
var zeroKey = zeroMap.keys().next().value;
var callbackCalls = 0;
var callbackThis = 1;
var callbackArgs = 0;
var callbackKey;
var callback = new Proxy(function(key) {
  "use strict";
  callbackCalls += 1;
  callbackThis = this;
  callbackArgs = arguments.length;
  callbackKey = key;
  return "symbol";
}, {});
var computedMiss = map.getOrInsertComputed(symbolKey, callback);
var computedHit = map.getOrInsertComputed(symbolKey, callback);
var presentInvalidThrows = false;
try {
  map.getOrInsertComputed(symbolKey, null);
} catch (error) {
  presentInvalidThrows = error instanceof TypeError;
}
var canonicalKey;
new Map().getOrInsertComputed(-0, function(key) { canonicalKey = key; });
[
  Map.prototype.getOrInsert.name,
  Map.prototype.getOrInsert.length,
  Map.prototype.getOrInsertComputed.name,
  Map.prototype.getOrInsertComputed.length,
  directMiss,
  directHit,
  map.get(objectKey),
  nanHit,
  zeroMiss,
  zeroMap.get(+0),
  1 / zeroKey === Infinity,
  computedMiss,
  computedHit,
  callbackCalls,
  callbackThis === undefined,
  callbackArgs,
  callbackKey === symbolKey,
  presentInvalidThrows,
  1 / canonicalKey === Infinity
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Map upsert methods should preserve keys and callback contracts");
        assert!(
            outcome.note.contains(
                "string(getOrInsert|2|getOrInsertComputed|2|object|object|object|nan|zero|zero|true|symbol|symbol|1|true|1|true|true|true)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_map_get_or_insert_computed_preserves_abrupt_and_reentrant_mutation() {
        let outcome = engine()
            .run_script(
                r#"
function CallbackError() {}
var callbackError = new CallbackError();
var map = new Map([[0, "zero"]]);
var abruptIdentity = false;
try {
  map.getOrInsertComputed(1, function() {
    map.set(0, "mutated");
    throw callbackError;
  });
} catch (error) {
  abruptIdentity = error === callbackError;
}
var absentAfterThrow = !map.has(1);
var sideEffectPreserved = map.get(0) === "mutated";
var overwriteResult = map.getOrInsertComputed(2, function() {
  map.set(2, "intermediate");
  return "final";
});
var overwriteStored = map.get(2) === "final";
var undefinedResult = map.getOrInsertComputed(3, function() {
  map.set(3, "intermediate");
});
var undefinedStored = map.has(3) && map.get(3) === undefined;
var other = __porfCreateRealm().global;
var brandRealm = false;
try {
  other.Map.prototype.getOrInsert.call({}, 1, 2);
} catch (error) {
  brandRealm = error instanceof other.TypeError && !(error instanceof TypeError);
}
var callbackRealm = false;
try {
  other.Map.prototype.getOrInsertComputed.call(new other.Map(), 1, null);
} catch (error) {
  callbackRealm = error instanceof other.TypeError && !(error instanceof TypeError);
}
[
  abruptIdentity,
  absentAfterThrow,
  sideEffectPreserved,
  overwriteResult,
  overwriteStored,
  undefinedResult === undefined,
  undefinedStored,
  brandRealm,
  callbackRealm
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Map computed upsert should preserve abrupt and reentrant mutation semantics");
        assert!(
            outcome
                .note
                .contains("string(true|true|true|final|true|true|true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_map_group_by_preserves_keys_order_and_callback_contract() {
        let outcome = engine()
            .run_script(
                r#"
var IntrinsicMap = Map;
var groupBy = Map.groupBy;
var invalidCallbackIteratorGets = 0;
var invalidCallbackThrows = false;
var invalidCallbackIterable = {};
Object.defineProperty(invalidCallbackIterable, Symbol.iterator, {
  get: function() {
    invalidCallbackIteratorGets += 1;
    throw new Error("iterator must not be read");
  }
});
try {
  groupBy(invalidCallbackIterable, null);
} catch (error) {
  invalidCallbackThrows = error instanceof TypeError;
}
var firstKey = {};
var secondKey = {};
var values = [firstKey, secondKey, firstKey];
var nextGets = 0;
var nextCalls = 0;
var iteratorMethod = new Proxy(function() {
  var index = 0;
  return {
    get next() {
      nextGets += 1;
      return new Proxy(function() {
        nextCalls += 1;
        if (index === values.length) return { done: true };
        var value = values[index];
        index += 1;
        return { done: false, value: value };
      }, {});
    }
  };
}, {});
var iterable = { [Symbol.iterator]: iteratorMethod };
var calls = [];
var callbackThis = 1;
var callback = new Proxy(function(value, index) {
  "use strict";
  callbackThis = this;
  calls.push((value === firstKey ? "first" : "second") + ":" + index);
  return value;
}, {});
var grouped = groupBy.call(function NotMap() {}, iterable, callback);
var zeroGrouped = groupBy([-0, +0], function(value) { return value; });
[
  typeof groupBy,
  groupBy.name,
  groupBy.length,
  invalidCallbackThrows,
  invalidCallbackIteratorGets,
  grouped instanceof IntrinsicMap,
  Object.getPrototypeOf(grouped) === IntrinsicMap.prototype,
  grouped.size,
  grouped.get(firstKey).length,
  grouped.get(firstKey)[0] === firstKey,
  grouped.get(firstKey)[1] === firstKey,
  grouped.get(secondKey)[0] === secondKey,
  calls.join(","),
  callbackThis === undefined,
  nextGets,
  nextCalls,
  zeroGrouped.size,
  zeroGrouped.has(+0),
  1 / zeroGrouped.keys().next().value === Infinity
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Map.groupBy should preserve Map keys and iterator callback contracts");
        assert!(
            outcome.note.contains(
                "string(function|groupBy|2|true|0|true|true|2|2|true|true|true|first:0,second:1,first:2|true|1|4|1|true|true)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_map_group_by_closes_only_for_callback_abrupt_completion() {
        let outcome = engine()
            .run_script(
                r#"
function CallbackError() {}
function NextError() {}
function DoneError() {}
function ValueError() {}
var closeCount = 0;
function iterableWithNext(next) {
  return {
    [Symbol.iterator]: function() {
      return {
        next: next,
        return: function() {
          closeCount += 1;
          throw new TypeError("suppressed close failure");
        }
      };
    }
  };
}
var callbackError = new CallbackError();
var callbackIdentity = false;
try {
  Map.groupBy(iterableWithNext(function() {
    return { done: false, value: 1 };
  }), new Proxy(function() { throw callbackError; }, {}));
} catch (error) {
  callbackIdentity = error === callbackError;
}
var callbackClosed = closeCount === 1;
var nextIdentity = false;
var nextError = new NextError();
try {
  Map.groupBy(iterableWithNext(function() { throw nextError; }), function() {});
} catch (error) {
  nextIdentity = error === nextError;
}
var nextDidNotClose = closeCount === 1;
var doneIdentity = false;
var doneError = new DoneError();
try {
  Map.groupBy(iterableWithNext(function() {
    return { get done() { throw doneError; } };
  }), function() {});
} catch (error) {
  doneIdentity = error === doneError;
}
var doneDidNotClose = closeCount === 1;
var valueIdentity = false;
var valueError = new ValueError();
try {
  Map.groupBy(iterableWithNext(function() {
    return { done: false, get value() { throw valueError; } };
  }), function() {});
} catch (error) {
  valueIdentity = error === valueError;
}
var valueDidNotClose = closeCount === 1;
var nonObjectThrows = false;
try {
  Map.groupBy(iterableWithNext(function() { return 1; }), function() {});
} catch (error) {
  nonObjectThrows = error instanceof TypeError;
}
[
  callbackIdentity,
  callbackClosed,
  nextIdentity,
  nextDidNotClose,
  doneIdentity,
  doneDidNotClose,
  valueIdentity,
  valueDidNotClose,
  nonObjectThrows,
  closeCount
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Map.groupBy should apply IteratorClose only to callback abrupt completion");
        assert!(
            outcome
                .note
                .contains("string(true|true|true|true|true|true|true|true|true|1)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_map_group_by_uses_the_method_defining_realm() {
        let outcome = engine()
            .run_script(
                r#"
var other = __porfCreateRealm().global;
var groupBy = other.Map.groupBy;
var grouped = groupBy.call(Map, [1, 2, 3], function(value) { return value % 2; });
var group = grouped.get(1);
[
  other.Map.groupBy !== Map.groupBy,
  Object.getPrototypeOf(grouped) === other.Map.prototype,
  grouped instanceof other.Map,
  grouped instanceof Map,
  group instanceof other.Array,
  group instanceof Array,
  group[0] === 1 && group[1] === 3
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Map.groupBy should allocate results from its defining realm intrinsics");
        assert!(
            outcome
                .note
                .contains("string(true|true|true|false|true|false|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_object_group_by_preserves_property_keys_and_callback_contract() {
        let outcome = engine()
            .run_script(
                r#"
var iteratorGets = 0;
var invalidCallbackThrows = false;
var invalidCallbackIterable = {};
Object.defineProperty(invalidCallbackIterable, Symbol.iterator, {
  get: function() {
    iteratorGets += 1;
    throw new Error("iterator must not be read");
  }
});
try {
  Object.groupBy(invalidCallbackIterable, null);
} catch (error) {
  invalidCallbackThrows = error instanceof TypeError;
}
var symbolKey = Symbol("group");
var sharedKey = { toString: function() { return "1"; } };
var values = ["symbol", "proto", "number", "object"];
var nextGets = 0;
var index = 0;
var iterable = {
  [Symbol.iterator]: new Proxy(function() {
    return {
      get next() {
        nextGets += 1;
        return new Proxy(function() {
          if (index === values.length) return { done: true };
          return { done: false, value: values[index++] };
        }, {});
      }
    };
  }, {})
};
var callbackThis = 1;
var callback = new Proxy(function(value, callbackIndex) {
  "use strict";
  callbackThis = this;
  if (callbackIndex === 0) return symbolKey;
  if (callbackIndex === 1) return "__proto__";
  if (callbackIndex === 2) return 1;
  return sharedKey;
}, {});
var grouped = Object.groupBy.call(function NotObject() {}, iterable, callback);
var keys = Object.keys(grouped);
[
  typeof Object.groupBy,
  Object.groupBy.name,
  Object.groupBy.length,
  invalidCallbackThrows,
  iteratorGets,
  Object.getPrototypeOf(grouped) === null,
  callbackThis === undefined,
  nextGets,
  grouped[symbolKey][0],
  Object.getOwnPropertySymbols(grouped)[0] === symbolKey,
  grouped.hasOwnProperty === undefined,
  Object.prototype.hasOwnProperty.call(grouped, "__proto__"),
  grouped.__proto__[0],
  grouped[1].join(","),
  keys.join(",")
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Object.groupBy should preserve property keys and iterator callback contracts");
        assert!(
            outcome.note.contains(
                "string(function|groupBy|2|true|0|true|true|1|symbol|true|true|true|proto|number,object|1,__proto__)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_object_group_by_closes_for_callback_and_property_key_abrupt_completion() {
        let outcome = engine()
            .run_script(
                r#"
function CallbackError() {}
function KeyError() {}
function NextError() {}
function DoneError() {}
function ValueError() {}
var closeCount = 0;
function iterableWithNext(next) {
  return {
    [Symbol.iterator]: function() {
      return {
        next: next,
        return: function() {
          closeCount += 1;
          throw new TypeError("suppressed close failure");
        }
      };
    }
  };
}
var callbackError = new CallbackError();
var callbackIdentity = false;
try {
  Object.groupBy(iterableWithNext(function() {
    return { done: false, value: 1 };
  }), function() { throw callbackError; });
} catch (error) {
  callbackIdentity = error === callbackError;
}
var keyError = new KeyError();
var keyIdentity = false;
try {
  Object.groupBy(iterableWithNext(function() {
    return { done: false, value: 1 };
  }), function() {
    return { toString: function() { throw keyError; } };
  });
} catch (error) {
  keyIdentity = error === keyError;
}
var nextError = new NextError();
var nextIdentity = false;
try {
  Object.groupBy(iterableWithNext(function() { throw nextError; }), function() {});
} catch (error) {
  nextIdentity = error === nextError;
}
var doneError = new DoneError();
var doneIdentity = false;
try {
  Object.groupBy(iterableWithNext(function() {
    return { get done() { throw doneError; } };
  }), function() {});
} catch (error) {
  doneIdentity = error === doneError;
}
var valueError = new ValueError();
var valueIdentity = false;
try {
  Object.groupBy(iterableWithNext(function() {
    return { done: false, get value() { throw valueError; } };
  }), function() {});
} catch (error) {
  valueIdentity = error === valueError;
}
[callbackIdentity, keyIdentity, nextIdentity, doneIdentity, valueIdentity, closeCount].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect(
                "Object.groupBy should close only after callback or property-key abrupt completion",
            );
        assert!(
            outcome.note.contains("string(true|true|true|true|true|2)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_object_group_by_uses_the_method_defining_realm() {
        let outcome = engine()
            .run_script(
                r#"
var other = __porfCreateRealm().global;
var grouped = other.Object.groupBy([1, 2, 3], function(value) { return value % 2; });
var group = grouped[1];
var realmError = false;
try {
  other.Object.groupBy([], null);
} catch (error) {
  realmError = error instanceof other.TypeError && !(error instanceof TypeError);
}
[
  other.Object.groupBy !== Object.groupBy,
  Object.getPrototypeOf(grouped) === null,
  group instanceof other.Array,
  group instanceof Array,
  group[0] === 1 && group[1] === 3,
  realmError
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Object.groupBy should allocate groups and errors from its defining realm");
        assert!(
            outcome
                .note
                .contains("string(true|true|true|false|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_map_constructor_and_methods_enforce_internal_slots() {
        let source = r#"
            var map = new Map(null);
            var undefinedMap = new Map(undefined);
            class DerivedMap extends Map {}
            var derived = new DerivedMap();
            var checks = [
                typeof Map,
                Object.getPrototypeOf(map) === Map.prototype,
                undefinedMap.size === 0,
                Object.getPrototypeOf(derived) === DerivedMap.prototype,
                map instanceof Map,
                map.size === 0
            ];
            try { Map(); checks.push(false); }
            catch (error) { checks.push(error instanceof TypeError); }
            try { Map.prototype.get.call({}); checks.push(false); }
            catch (error) { checks.push(error instanceof TypeError); }
            try {
                Object.getOwnPropertyDescriptor(Map.prototype, 'size').get.call([]);
                checks.push(false);
            } catch (error) { checks.push(error instanceof TypeError); }
            checks.join('|');
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Map constructor and methods should enforce their runtime contracts");
        assert!(
            outcome
                .note
                .contains("string(function|true|true|true|true|true|true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_map_constructs_from_entry_iterables() {
        let outcome = engine()
            .run_script(
                "var map = new Map([[1, 'a'], [2, 'b'], [1, 'c']]); var stringThrows = false; try { new Map('a'); } catch (error) { stringThrows = error instanceof TypeError; } [map.size, map.get(1), map.get(2), stringThrows].join('|');",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Map should consume entry iterables and reject primitive entries");
        assert!(
            outcome.note.contains("string(2|c|b|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_map_captures_setter_before_acquiring_iterator() {
        let outcome = engine()
            .run_script(
                r#"
var events = [];
var intrinsicSet = Map.prototype.set;
class DerivedMap extends Map {}
Object.defineProperty(DerivedMap.prototype, "set", {
  configurable: true,
  get: function() {
    events.push("get set");
    return function(key, value) {
      events.push("set " + key + ":" + value);
      return intrinsicSet.call(this, key, value);
    };
  }
});
var index = 0;
var iterable = {};
Object.defineProperty(iterable, Symbol.iterator, {
  get: function() {
    events.push("get iterator");
    return function() {
      events.push("call iterator");
      return {
        get next() {
          events.push("get next");
          return function() {
            events.push("next");
            if (index === 2) return { done: true };
            index += 1;
            var entryIndex = index;
            return {
              done: false,
              value: {
                get 0() { events.push("get 0"); return entryIndex; },
                get 1() { events.push("get 1"); return entryIndex === 1 ? "a" : "b"; }
              }
            };
          };
        }
      };
    };
  }
});
var map = new DerivedMap(iterable);
events.join("|") + ":" + map.size;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Map should observe constructor iteration in specification order");
        assert!(
            outcome.note.contains(
                "string(get set|get iterator|call iterator|get next|next|get 0|get 1|set 1:a|next|get 0|get 1|set 2:b|next:2)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_map_closes_only_after_entry_or_set_abrupt_completions() {
        let outcome = engine()
            .run_script(
                r#"
function NextError() {}
function ValueError() {}
function FirstError() {}
function SecondError() {}
function SetError() {}
var closeCount = 0;
function iterableWithNext(next) {
  return {
    [Symbol.iterator]: function() {
      return {
        next: next,
        return: function() {
          closeCount += 1;
          throw new TypeError("suppressed close failure");
        }
      };
    }
  };
}
var nextErrorPreserved = false;
try {
  new Map(iterableWithNext(function() { throw new NextError(); }));
} catch (error) {
  nextErrorPreserved = error instanceof NextError;
}
var nextDidNotClose = closeCount === 0;
var valueErrorPreserved = false;
try {
  new Map(iterableWithNext(function() {
    return { done: false, get value() { throw new ValueError(); } };
  }));
} catch (error) {
  valueErrorPreserved = error instanceof ValueError;
}
var valueDidNotClose = closeCount === 0;
var nonObjectRejected = false;
try {
  new Map(iterableWithNext(function() { return { done: false, value: 1 }; }));
} catch (error) {
  nonObjectRejected = error instanceof TypeError;
}
var firstErrorPreserved = false;
try {
  new Map(iterableWithNext(function() {
    return { done: false, value: { get 0() { throw new FirstError(); } } };
  }));
} catch (error) {
  firstErrorPreserved = error instanceof FirstError;
}
var secondErrorPreserved = false;
try {
  new Map(iterableWithNext(function() {
    return { done: false, value: { 0: 1, get 1() { throw new SecondError(); } } };
  }));
} catch (error) {
  secondErrorPreserved = error instanceof SecondError;
}
var intrinsicSet = Map.prototype.set;
Map.prototype.set = function() { throw new SetError(); };
var setErrorPreserved = false;
try {
  new Map(iterableWithNext(function() {
    return { done: false, value: [1, 2] };
  }));
} catch (error) {
  setErrorPreserved = error instanceof SetError;
}
Map.prototype.set = intrinsicSet;
[nextErrorPreserved, nextDidNotClose, valueErrorPreserved, valueDidNotClose,
 nonObjectRejected, firstErrorPreserved, secondErrorPreserved, setErrorPreserved,
 closeCount].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Map should close only after entry processing or its captured setter throws");
        assert!(
            outcome
                .note
                .contains("string(true|true|true|true|true|true|true|true|4)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_map_accepts_callable_proxy_iterator_components() {
        let outcome = engine()
            .run_script(
                r#"
var intrinsicSet = Map.prototype.set;
var setCalls = 0;
var proxiedSet = new Proxy(function(key, value) {
  setCalls += 1;
  return intrinsicSet.call(this, key, value);
}, {});
class DerivedMap extends Map {}
DerivedMap.prototype.set = proxiedSet;
var index = 0;
var proxiedNext = new Proxy(function() {
  if (index === 2) return { done: true };
  index += 1;
  return { done: false, value: new Proxy([index, index * 10], {}) };
}, {});
var proxiedIterator = new Proxy(function() {
  return { next: proxiedNext };
}, {});
var iterable = { [Symbol.iterator]: proxiedIterator };
var map = new DerivedMap(iterable);
[map.size, map.get(1), map.get(2), setCalls].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Map should call Proxy-wrapped iterator components and setter");
        assert!(
            outcome.note.contains("string(2|10|20|2)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_map_for_each_observes_callback_contracts_and_mutations() {
        let source = r#"
var context = {};
var calls = [];
var map = new Map([["a", 1], ["b", 2], ["c", 3]]);
var callback = new Proxy(function(value, key, receiver) {
  calls.push(key + ":" + value + ":" + (receiver === map) + ":" + (this === context));
  if (key === "a") {
    map.delete("b");
    map.set("d", 4);
  }
  if (key === "c" && value === 3) {
    map.delete("c");
    map.set("c", 30);
  }
}, {});
var result = map.forEach(callback, context);
var cleared = new Map([["x", 1], ["y", 2]]);
var clearCalls = [];
cleared.forEach(function(value, key) {
  clearCalls.push(key);
  if (key === "x") {
    cleared.clear();
    cleared.set("z", 3);
  }
});
[result === undefined, calls.join(","), clearCalls.join(",")].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Map.prototype.forEach should observe callback and mutation semantics");
        assert!(
            outcome.note.contains(
                "string(true|a:1:true:true,c:3:true:true,d:4:true:true,c:30:true:true|x,z)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_map_for_each_checks_brands_and_preserves_callback_throws() {
        let source = r#"
var brandThrows = false;
var callbackThrows = false;
try { Map.prototype.forEach.call({}, function() {}); } catch (error) {
  brandThrows = error instanceof TypeError;
}
try { new Map().forEach(null); } catch (error) {
  callbackThrows = error instanceof TypeError;
}
var expected = new Error("map callback");
var actual;
try {
  new Map([[1, 2]]).forEach(new Proxy(function() { throw expected; }, {}));
} catch (error) {
  actual = error;
}
[brandThrows, callbackThrows, actual === expected].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Map.prototype.forEach should enforce brands and preserve callback throws");
        assert!(
            outcome.note.contains("string(true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_map_iterators_are_live_and_remain_exhausted() {
        let source = r#"
var map = new Map([["a", 1], ["b", 2]]);
var keys = map.keys();
var first = keys.next();
map.delete("b");
map.set("c", 3);
var second = keys.next();
var done = keys.next();
map.set("d", 4);
var remainsDone = keys.next();
var entry = map.entries().next().value;
[
  first.value === "a" && first.done === false,
  second.value === "c" && second.done === false,
  done.done === true,
  remainsDone.done === true,
  entry[0] === "a" && entry[1] === 1
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Map iterators should preserve insertion order and live mutation semantics");
        assert!(
            outcome.note.contains("string(true|true|true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_map_iterators_have_spec_surface_and_brand_checks() {
        let source = r#"
var iterator = new Map([[1, 2]]).values();
var prototype = Object.getPrototypeOf(iterator);
var brandThrows = false;
try { prototype.next.call({}); } catch (error) { brandThrows = error instanceof TypeError; }
[
  Map.prototype.entries === Map.prototype[Symbol.iterator],
  iterator[Symbol.iterator]() === iterator,
  Object.getPrototypeOf(prototype)[Symbol.iterator].call(iterator) === iterator,
  Object.prototype.toString.call(iterator) === "[object Map Iterator]",
  brandThrows
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Map iterators should expose the dedicated iterator prototype surface");
        assert!(
            outcome.note.contains("string(true|true|true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_map_uses_cross_realm_new_target_prototype_fallback() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let otherConstructor = other.Object;
otherConstructor.prototype = null;
let direct = Reflect.construct(Map, [], otherConstructor);
let proxied = Reflect.construct(Map, [], new Proxy(otherConstructor, {}));
let boundConstructed = Reflect.construct(Map, [], otherConstructor.bind(null));
let revocable;
revocable = Proxy.revocable(otherConstructor, {
  get(target, key, receiver) {
    if (key === "prototype") {
      revocable.revoke();
      return null;
    }
    return Reflect.get(target, key, receiver);
  }
});
let revokedThrows = false;
try {
  Reflect.construct(Map, [], revocable.proxy);
} catch (error) {
  revokedThrows = error instanceof TypeError;
}
[
  other.Map.prototype !== Map.prototype,
  Object.getPrototypeOf(direct) === other.Map.prototype,
  Object.getPrototypeOf(proxied) === other.Map.prototype,
  Object.getPrototypeOf(boundConstructed) === other.Map.prototype,
  Object.getPrototypeOf(direct) !== Map.prototype,
  revokedThrows
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Map construction should use the original newTarget function realm");
        assert!(
            outcome
                .note
                .contains("string(true|true|true|true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_set_core_preserves_same_value_zero_and_insertion_updates() {
        for (source, expected) in [
            (
                "var set = new Set(); var object = {}; var symbol = Symbol('k'); set.add('x').add(NaN).add(-0).add(object).add(symbol).add(900719925474099100000n); set.add('x').add(+0).add(900719925474099100000n).add(900719925474099100000); [set.size, set.has('x'), set.has(NaN), set.has(+0), set.has(object), set.has(symbol), set.has(900719925474099100000n), set.has(900719925474099100000), set.add(1) === set].join('|');",
                "string(7|true|true|true|true|true|true|true|true)",
            ),
            (
                "var set = new Set(); for (var i = 0; i < 9; i += 1) set.add(i); set.delete(2); set.delete(7); set.add(2); [set.size, set.has(2), set.has(7), set.delete(7), set.delete(8), set.size].join('|');",
                "string(8|true|false|false|true|7)",
            ),
            (
                "var set = new Set(); set.add(1).add(2); set.clear(); set.add(3); [set.size, set.has(1), set.has(3)].join('|');",
                "string(1|false|true)",
            ),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| panic!("Set core case should run for `{source}`: {err:?}"));
            assert!(
                outcome.note.contains(expected),
                "source: {source}, note: {}",
                outcome.note
            );
        }
    }

    #[test]
    fn wasm_backend_set_algebra_preserves_values_order_and_intrinsic_construction() {
        let outcome = engine()
            .run_script(
                r#"
var left = new Set([1, 2, 3, -0]);
var right = new Set([3, 4, +0]);
var originalAdd = Set.prototype.add;
Set.prototype.add = function() { throw new Error("Set algebra must not call add"); };
var difference = left.difference(right);
var intersection = left.intersection(right);
var symmetricDifference = left.symmetricDifference(right);
var union = left.union(right);
Set.prototype.add = originalAdd;
var mapIntersection = left.intersection(new Map([[2, "two"], [4, "four"]]));
var zeroIterator = union.values();
zeroIterator.next();
zeroIterator.next();
zeroIterator.next();
var zero = zeroIterator.next().value;
var notConstructors = true;
for (var method of [
  Set.prototype.difference,
  Set.prototype.intersection,
  Set.prototype.symmetricDifference,
  Set.prototype.union
]) {
  try { new method(new Set()); notConstructors = false; } catch (error) {
    notConstructors = notConstructors && error instanceof TypeError;
  }
}
[
  Set.prototype.difference.name,
  Set.prototype.difference.length,
  Set.prototype.intersection.name,
  Set.prototype.intersection.length,
  Set.prototype.symmetricDifference.name,
  Set.prototype.symmetricDifference.length,
  Set.prototype.union.name,
  Set.prototype.union.length,
  [...difference].join(","),
  [...intersection].join(","),
  [...symmetricDifference].join(","),
  [...union].join(","),
  [...mapIntersection].join(","),
  difference instanceof Set,
  Object.getPrototypeOf(union) === Set.prototype,
  union.size,
  union.has(-0) && union.has(+0),
  1 / zero === Infinity,
  left.size,
  right.size,
  notConstructors
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set algebra should preserve values, ordering, and intrinsic construction");
        assert!(
            outcome.note.contains(
                "string(difference|1|intersection|1|symmetricDifference|1|union|1|1,2|3,0|1,2,4|1,2,3,0,4|2|true|true|5|true|true|4|3|true)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_set_algebra_observes_set_like_order_mutation_and_realms() {
        let outcome = engine()
            .run_script(
                r#"
var order = [];
var differenceLike = {
  get size() {
    order.push("get size");
    return { valueOf: function() { order.push("ToNumber size"); return 3; } };
  },
  get has() {
    order.push("get has");
    return new Proxy(function(value) { order.push("call has " + value); return value === 1; }, {});
  },
  get keys() {
    order.push("get keys");
    return function() { throw new Error("keys must not be called"); };
  }
};
var difference = new Set([1, 2]).difference(differenceLike);
var differenceOrder = order.join(",");

order = [];
var values = [-0, 2, 3];
var index = 0;
var keys = new Proxy(function() {
  order.push("call keys");
  return {
    get next() {
      order.push("get next");
      return new Proxy(function() {
        order.push("call next");
        return index === values.length
          ? { get done() { order.push("get done"); return true; } }
          : {
              get done() { order.push("get done"); return false; },
              get value() { order.push("get value"); return values[index++]; }
            };
      }, {});
    }
  };
}, {});
var unionLike = {
  get size() { order.push("get size"); return 3; },
  get has() { order.push("get has"); return function() { throw new Error("has must not be called"); }; },
  get keys() { order.push("get keys"); return keys; }
};
var union = new Set([1, 2]).union(unionLike);
var unionOrder = order.join(",");

var base = new Set(["a", "b", "c", "d", "e"]);
var mutationIndex = 0;
var mutationValues = ["x", "b", "c", "c"];
var mutationLike = {
  size: 4,
  get has() { base.add("q"); return function() { throw new Error("has must not be called"); }; },
  keys: function() {
    return { next: function() {
      if (mutationIndex === 0) {
        base.delete("b"); base.delete("c"); base.add("b"); base.add("d");
      }
      return mutationIndex === mutationValues.length
        ? { done: true }
        : { done: false, value: mutationValues[mutationIndex++] };
    } };
  }
};
var symmetric = base.symmetricDifference(mutationLike);

function NextError() {}
var nextError = new NextError();
var closeCount = 0;
var abruptIdentity = false;
try {
  new Set().union({
    size: 0,
    has: function() {},
    keys: function() {
      return {
        next: function() { throw nextError; },
        return: function() { closeCount += 1; return {}; }
      };
    }
  });
} catch (error) { abruptIdentity = error === nextError; }

var other = __porfCreateRealm().global;
var realmResult = other.Set.prototype.union.call(new other.Set([1]), new Set([2]));
var brandRealm = false;
try { other.Set.prototype.difference.call({}, new Set()); } catch (error) {
  brandRealm = error instanceof other.TypeError && !(error instanceof TypeError);
}
var sizeTypeRealm = false;
try {
  other.Set.prototype.union.call(new other.Set(), { size: NaN, has: function() {}, keys: function() {} });
} catch (error) { sizeTypeRealm = error instanceof other.TypeError && !(error instanceof TypeError); }
var sizeRangeRealm = false;
try {
  other.Set.prototype.union.call(new other.Set(), { size: -1, has: function() {}, keys: function() {} });
} catch (error) { sizeRangeRealm = error instanceof other.RangeError && !(error instanceof RangeError); }
[
  [...difference].join(","),
  differenceOrder,
  [...union].join(","),
  unionOrder,
  [...symmetric].join(","),
  [...base].join(","),
  abruptIdentity,
  closeCount,
  Object.getPrototypeOf(realmResult) === other.Set.prototype,
  realmResult instanceof other.Set,
  realmResult instanceof Set,
  brandRealm,
  sizeTypeRealm,
  sizeRangeRealm
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set algebra should preserve observable set-like and realm semantics");
        assert!(
            outcome.note.contains(
                "string(2|get size,ToNumber size,get has,get keys,call has 1,call has 2|1,2,0,3|get size,get has,get keys,call keys,get next,call next,get done,get value,call next,get done,get value,call next,get done,get value,call next,get done|a,c,d,e,q,x|a,d,e,q,b|true|0|true|true|false|true|true|true)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_set_algebra_converts_set_like_size_to_integer_or_infinity() {
        let outcome = engine()
            .run_script(
                r#"
var result = new Set([1]).difference({
  size: -0.5,
  has: function() { throw new Error("has must not be called"); },
  keys: function() { return new Set().keys(); }
});
[...result].join(",");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set algebra should convert a set-like size before checking its range");
        assert!(outcome.note.contains("string(1)"), "note: {}", outcome.note);
    }

    #[test]
    fn wasm_backend_set_predicates_compare_values_and_expose_spec_function_shapes() {
        let outcome = engine()
            .run_script(
                r#"
var left = new Set([1, 2, NaN, -0]);
var subset = new Set([2, +0]);
var notConstructors = true;
for (var method of [
  Set.prototype.isDisjointFrom,
  Set.prototype.isSubsetOf,
  Set.prototype.isSupersetOf
]) {
  try { new method(new Set()); notConstructors = false; } catch (error) {
    notConstructors = notConstructors && error instanceof TypeError;
  }
}
[
  Set.prototype.isDisjointFrom.name,
  Set.prototype.isDisjointFrom.length,
  Set.prototype.isSubsetOf.name,
  Set.prototype.isSubsetOf.length,
  Set.prototype.isSupersetOf.name,
  Set.prototype.isSupersetOf.length,
  left.isDisjointFrom(new Set([3, 4])),
  left.isDisjointFrom(new Set([4, NaN])),
  subset.isSubsetOf(left),
  left.isSubsetOf(subset),
  left.isSupersetOf(subset),
  subset.isSupersetOf(left),
  left.isSupersetOf(new Map([[1, "one"], [2, "two"]])),
  notConstructors
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set predicates should compare values with Set semantics");
        assert!(
            outcome.note.contains(
                "string(isDisjointFrom|1|isSubsetOf|1|isSupersetOf|1|true|false|true|false|true|false|true|true)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_set_predicates_observe_set_like_order_proxies_and_live_mutation() {
        let outcome = engine()
            .run_script(
                r#"
var order = [];
var setLike = {
  get size() { order.push("get size"); return { valueOf: function() { order.push("ToNumber size"); return 3; } }; },
  get has() { order.push("get has"); return new Proxy(function(value) { order.push("call has " + value); return value === "b"; }, {}); },
  get keys() { order.push("get keys"); return function() { throw new Error("keys must not be called"); }; }
};
var disjoint = new Set(["x", "b"]).isDisjointFrom(setLike);
var disjointOrder = order.join(",");

order = [];
var index = 0;
var keys = new Proxy(function() {
  order.push("call keys");
  return {
    get next() {
      order.push("get next");
      return new Proxy(function() {
        order.push("call next");
        return index === 2
          ? { get done() { order.push("get done"); return true; } }
          : {
              get done() { order.push("get done"); return false; },
              get value() { order.push("get value"); return ["a", "b"][index++]; }
            };
      }, {});
    }
  };
}, {});
var supersetLike = {
  get size() { order.push("get size"); return 2; },
  get has() { order.push("get has"); return function() { throw new Error("has must not be called"); }; },
  get keys() { order.push("get keys"); return keys; }
};
var superset = new Set(["a", "b", "c"]).isSupersetOf(supersetLike);
var supersetOrder = order.join(",");

var base = new Set(["a", "b", "c"]);
var mutationLike = {
  size: 3,
  has: function(value) {
    if (value === "a") {
      base.delete("b");
      base.delete("c");
      base.add("b");
    }
    return false;
  },
  keys: function() { throw new Error("keys must not be called"); }
};
var liveDisjoint = base.isDisjointFrom(mutationLike);

var supersetBase = new Set(["a", "b", "c"]);
var mutationIndex = 0;
var supersetMutationLike = {
  size: 2,
  has: function() { throw new Error("has must not be called"); },
  keys: function() { return { next: function() {
    if (mutationIndex === 0) {
      supersetBase.delete("b");
      supersetBase.delete("c");
      supersetBase.add("b");
    }
    return mutationIndex === 2
      ? { done: true }
      : { done: false, value: ["a", "b"][mutationIndex++] };
  } }; }
};
var liveSuperset = supersetBase.isSupersetOf(supersetMutationLike);
[
  disjoint,
  disjointOrder,
  superset,
  supersetOrder,
  liveDisjoint,
  [...base].join(","),
  liveSuperset,
  [...supersetBase].join(",")
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set predicates should preserve observable set-like operations");
        assert!(
            outcome.note.contains(
                "string(false|get size,ToNumber size,get has,get keys,call has x,call has b|true|get size,get has,get keys,call keys,get next,call next,get done,get value,call next,get done,get value,call next,get done|true|a,b|true|a,b)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_set_predicates_close_early_iterators_and_preserve_realms_and_abrupt_values() {
        let outcome = engine()
            .run_script(
                r#"
var closeCount = 0;
function earlyIterator(value, returnMethod) {
  return {
    next: function() { return { done: false, value: value }; },
    return: returnMethod
  };
}
var disjoint = new Set([1, 2]).isDisjointFrom({
  size: 1,
  has: function() {},
  keys: function() { return earlyIterator(2, function() { closeCount += 1; return {}; }); }
});
var superset = new Set([1, 2]).isSupersetOf({
  size: 1,
  has: function() {},
  keys: function() { return earlyIterator(3, function() { closeCount += 1; return {}; }); }
});

var closeError = {};
var closeIdentity = false;
try {
  new Set([1]).isSupersetOf({
    size: 1,
    has: function() {},
    keys: function() { return earlyIterator(2, function() { throw closeError; }); }
  });
} catch (error) { closeIdentity = error === closeError; }

var nextError = {};
var nextIdentity = false;
var closeAfterNextThrow = 0;
try {
  new Set([1]).isSupersetOf({
    size: 1,
    has: function() {},
    keys: function() { return {
      next: function() { throw nextError; },
      return: function() { closeAfterNextThrow += 1; return {}; }
    }; }
  });
} catch (error) { nextIdentity = error === nextError; }

var hasError = {};
var hasIdentity = false;
try {
  new Set([1]).isSubsetOf({
    size: 1,
    has: function() { throw hasError; },
    keys: function() {}
  });
} catch (error) { hasIdentity = error === hasError; }

var closeResultTypeError = false;
try {
  new Set([1]).isSupersetOf({
    size: 1,
    has: function() {},
    keys: function() { return earlyIterator(2, function() { return 1; }); }
  });
} catch (error) { closeResultTypeError = error instanceof TypeError; }

var other = __porfCreateRealm().global;
var realmResult = other.Set.prototype.isSubsetOf.call(new other.Set([1]), new Set([1, 2]));
var brandRealm = false;
try { other.Set.prototype.isSubsetOf.call({}, new Set()); } catch (error) {
  brandRealm = error instanceof other.TypeError && !(error instanceof TypeError);
}
var sizeRangeRealm = false;
try {
  other.Set.prototype.isSupersetOf.call(new other.Set(), {
    size: -1,
    has: function() {},
    keys: function() {}
  });
} catch (error) { sizeRangeRealm = error instanceof other.RangeError && !(error instanceof RangeError); }
[
  disjoint,
  superset,
  closeCount,
  closeIdentity,
  nextIdentity,
  hasIdentity,
  closeAfterNextThrow,
  closeResultTypeError,
  realmResult,
  brandRealm,
  sizeRangeRealm
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set predicates should close early iterators and preserve abrupt values");
        assert!(
            outcome
                .note
                .contains("string(false|false|2|true|true|true|0|true|true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_set_constructor_and_methods_enforce_internal_slots() {
        let source = r#"
            var set = new Set(null);
            var undefinedSet = new Set(undefined);
            class DerivedSet extends Set {}
            var derived = new DerivedSet();
            var checks = [
                typeof Set,
                Object.getPrototypeOf(set) === Set.prototype,
                undefinedSet.size === 0,
                Object.getPrototypeOf(derived) === DerivedSet.prototype,
                set instanceof Set,
                set.size === 0
            ];
            try { Set(); checks.push(false); }
            catch (error) { checks.push(error instanceof TypeError); }
            try { Set.prototype.add.call({}); checks.push(false); }
            catch (error) { checks.push(error instanceof TypeError); }
            try { Set.prototype.has.call(new Map()); checks.push(false); }
            catch (error) { checks.push(error instanceof TypeError); }
            try {
                Object.getOwnPropertyDescriptor(Set.prototype, 'size').get.call([]);
                checks.push(false);
            } catch (error) { checks.push(error instanceof TypeError); }
            checks.join('|');
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set constructor and methods should enforce their runtime contracts");
        assert!(
            outcome
                .note
                .contains("string(function|true|true|true|true|true|true|true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_set_constructs_from_array_and_string_iterables() {
        let outcome = engine()
            .run_script(
                "var numbers = new Set([1, 2, 2, -0]); var letters = new Set('aba'); [numbers.size, numbers.has(0), letters.size, letters.has('a'), letters.has('b')].join('|');",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set should consume array and string iterables");
        assert!(
            outcome.note.contains("string(3|true|2|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_set_captures_adder_before_acquiring_iterator() {
        let outcome = engine()
            .run_script(
                r#"
var events = [];
var intrinsicAdd = Set.prototype.add;
class DerivedSet extends Set {}
Object.defineProperty(DerivedSet.prototype, "add", {
  configurable: true,
  get: function() {
    events.push("get add");
    return function(value) {
      events.push("add " + value);
      return intrinsicAdd.call(this, value);
    };
  }
});
var index = 0;
var iterable = {};
Object.defineProperty(iterable, Symbol.iterator, {
  get: function() {
    events.push("get iterator");
    return function() {
      events.push("call iterator");
      return {
        get next() {
          events.push("get next");
          return function() {
            events.push("next");
            if (index === 2) return { done: true };
            index += 1;
            return {
              done: false,
              get value() {
                events.push("value");
                return index;
              }
            };
          };
        }
      };
    };
  }
});
var set = new DerivedSet(iterable);
events.join("|") + ":" + set.size;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set should observe constructor iteration in specification order");
        assert!(
            outcome.note.contains(
                "string(get add|get iterator|call iterator|get next|next|value|add 1|next|value|add 2|next:2)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_set_closes_only_after_abrupt_adder_calls() {
        let outcome = engine()
            .run_script(
                r#"
function NextError() {}
function ValueError() {}
function AddError() {}
var closeCount = 0;
function iterableWithNext(next) {
  return {
    [Symbol.iterator]: function() {
      return {
        next: next,
        return: function() {
          closeCount += 1;
          throw new TypeError("suppressed close failure");
        }
      };
    }
  };
}
var nextErrorPreserved = false;
try {
  new Set(iterableWithNext(function() { throw new NextError(); }));
} catch (error) {
  nextErrorPreserved = error instanceof NextError;
}
var valueErrorPreserved = false;
try {
  new Set(iterableWithNext(function() {
    return { done: false, get value() { throw new ValueError(); } };
  }));
} catch (error) {
  valueErrorPreserved = error instanceof ValueError;
}
var intrinsicAdd = Set.prototype.add;
Set.prototype.add = function() { throw new AddError(); };
var addErrorPreserved = false;
try {
  new Set(iterableWithNext(function() { return { done: false, value: 1 }; }));
} catch (error) {
  addErrorPreserved = error instanceof AddError;
}
Set.prototype.add = intrinsicAdd;
[nextErrorPreserved, valueErrorPreserved, addErrorPreserved, closeCount].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set should close an iterator only after its captured adder throws");
        assert!(
            outcome.note.contains("string(true|true|true|1)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_set_accepts_callable_proxy_iterator_components() {
        let outcome = engine()
            .run_script(
                r#"
var intrinsicAdd = Set.prototype.add;
var addCalls = 0;
var proxiedAdd = new Proxy(function(value) {
  addCalls += 1;
  return intrinsicAdd.call(this, value);
}, {});
class DerivedSet extends Set {}
DerivedSet.prototype.add = proxiedAdd;
var index = 0;
var proxiedNext = new Proxy(function() {
  if (index === 2) return { done: true };
  index += 1;
  return { done: false, value: index };
}, {});
var proxiedIterator = new Proxy(function() {
  return { next: proxiedNext };
}, {});
var iterable = { [Symbol.iterator]: proxiedIterator };
var set = new DerivedSet(iterable);
[set.size, set.has(1), set.has(2), addCalls].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set should call Proxy-wrapped iterator components");
        assert!(
            outcome.note.contains("string(2|true|true|2)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_set_for_each_observes_callback_contracts_and_mutations() {
        let source = r#"
var context = {};
var calls = [];
var set = new Set(["a", "b", "c"]);
var callback = new Proxy(function(value, key, receiver) {
  calls.push(value + ":" + (key === value) + ":" + (receiver === set) + ":" + (this === context));
  if (value === "a") {
    set.delete("b");
    set.add("d");
  }
  if (value === "c") {
    set.delete("c");
    set.add("C");
  }
}, {});
var result = set.forEach(callback, context);
var cleared = new Set(["x", "y"]);
var clearCalls = [];
cleared.forEach(function(value) {
  clearCalls.push(value);
  if (value === "x") {
    cleared.clear();
    cleared.add("z");
  }
});
[result === undefined, calls.join(","), clearCalls.join(",")].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set.prototype.forEach should observe callback and mutation semantics");
        assert!(
            outcome.note.contains(
                "string(true|a:true:true:true,c:true:true:true,d:true:true:true,C:true:true:true|x,z)"
            ),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_set_for_each_checks_brands_and_preserves_callback_throws() {
        let source = r#"
var brandThrows = false;
var callbackThrows = false;
try { Set.prototype.forEach.call({}, function() {}); } catch (error) {
  brandThrows = error instanceof TypeError;
}
try { new Set().forEach(null); } catch (error) {
  callbackThrows = error instanceof TypeError;
}
var expected = new Error("set callback");
var actual;
try {
  new Set([1]).forEach(new Proxy(function() { throw expected; }, {}));
} catch (error) {
  actual = error;
}
[brandThrows, callbackThrows, actual === expected].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set.prototype.forEach should enforce brands and preserve callback throws");
        assert!(
            outcome.note.contains("string(true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_set_iterators_are_live_and_remain_exhausted() {
        let source = r#"
var set = new Set(["a", "b"]);
var values = set.values();
var first = values.next();
set.delete("b");
set.add("c");
var second = values.next();
var done = values.next();
set.add("d");
var remainsDone = values.next();
var entry = set.entries().next().value;
[
  first.value === "a" && first.done === false,
  second.value === "c" && second.done === false,
  done.done === true,
  remainsDone.done === true,
  entry[0] === "a" && entry[1] === "a"
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set iterators should preserve insertion order and live mutation semantics");
        assert!(
            outcome.note.contains("string(true|true|true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_set_iterators_have_spec_surface_and_brand_checks() {
        let source = r#"
var iterator = new Set([1]).values();
var prototype = Object.getPrototypeOf(iterator);
var brandThrows = false;
try { prototype.next.call({}); } catch (error) { brandThrows = error instanceof TypeError; }
[
  Set.prototype.values === Set.prototype.keys,
  Set.prototype.values === Set.prototype[Symbol.iterator],
  iterator[Symbol.iterator]() === iterator,
  Object.getPrototypeOf(prototype)[Symbol.iterator].call(iterator) === iterator,
  Object.prototype.toString.call(iterator) === "[object Set Iterator]",
  brandThrows
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set iterators should expose the dedicated iterator prototype surface");
        assert!(
            outcome
                .note
                .contains("string(true|true|true|true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_collection_iterators_use_the_method_defining_realm() {
        let source = r#"
var other = __porfCreateRealm().global;
var mapIterator = other.Map.prototype.keys.call(new other.Map([[1, 2]]));
var setIterator = other.Set.prototype.values.call(new other.Set([1]));
[
  Object.getPrototypeOf(mapIterator) !== Object.getPrototypeOf(new Map().keys()),
  Object.getPrototypeOf(setIterator) !== Object.getPrototypeOf(new Set().values()),
  other.Map.prototype.entries === other.Map.prototype[other.Symbol.iterator],
  other.Set.prototype.values === other.Set.prototype.keys,
  other.Set.prototype.values === other.Set.prototype[other.Symbol.iterator],
  mapIterator.next().value === 1,
  setIterator.next().value === 1
].join("|");
"#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("collection iterators should use their method defining realm prototypes");
        assert!(
            outcome
                .note
                .contains("string(true|true|true|true|true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_set_uses_cross_realm_new_target_prototype_fallback() {
        let outcome = engine()
            .run_script(
                r#"
let other = __porfCreateRealm().global;
let otherConstructor = other.Object;
otherConstructor.prototype = null;
let direct = Reflect.construct(Set, [], otherConstructor);
let proxied = Reflect.construct(Set, [], new Proxy(otherConstructor, {}));
let boundConstructed = Reflect.construct(Set, [], otherConstructor.bind(null));
let revocable;
revocable = Proxy.revocable(otherConstructor, {
  get(target, key, receiver) {
    if (key === "prototype") {
      revocable.revoke();
      return null;
    }
    return Reflect.get(target, key, receiver);
  }
});
let revokedThrows = false;
try {
  Reflect.construct(Set, [], revocable.proxy);
} catch (error) {
  revokedThrows = error instanceof TypeError;
}
[
  other.Set.prototype !== Set.prototype,
  Object.getPrototypeOf(direct) === other.Set.prototype,
  Object.getPrototypeOf(proxied) === other.Set.prototype,
  Object.getPrototypeOf(boundConstructed) === other.Set.prototype,
  Object.getPrototypeOf(direct) !== Set.prototype,
  revokedThrows
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set construction should use the original newTarget function realm");
        assert!(
            outcome
                .note
                .contains("string(true|true|true|true|true|true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_promise_constructor_calls_executor_synchronously() {
        let source = r#"
            let resolve;
            let reject;
            let calls = 0;
            let promise = new Promise(function (onFulfilled, onRejected) {
                calls += 1;
                resolve = onFulfilled;
                reject = onRejected;
            });
            calls === 1
                && typeof promise === "object"
                && Object.getPrototypeOf(promise) === Promise.prototype
                && promise.constructor === Promise
                && typeof resolve === "function"
                && typeof reject === "function"
                && resolve !== reject
                && resolve.length === 1
                && reject.length === 1
                && resolve(42) === undefined
                && reject(9) === undefined;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise constructor should run: {err:?}"));
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_promise_constructor_rejects_invalid_invocations() {
        let source = r#"
            let nonCallableThrows = false;
            let missingNewThrows = false;
            let prototypeAccessed = false;
            let orderIsCorrect = false;
            try { new Promise(1); } catch (error) {
                nonCallableThrows = error instanceof TypeError;
            }
            try { Promise(function () {}); } catch (error) {
                missingNewThrows = error instanceof TypeError;
            }
            let newTarget = (function () {}).bind();
            Object.defineProperty(newTarget, "prototype", {
                get: function () {
                    prototypeAccessed = true;
                    throw new Error();
                }
            });
            try { Reflect.construct(Promise, [], newTarget); } catch (error) {
                orderIsCorrect = error instanceof TypeError && !prototypeAccessed;
            }
            nonCallableThrows && missingNewThrows && orderIsCorrect;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("invalid Promise constructor uses should run: {err:?}"));
        assert!(
            outcome.note.contains("boolean(true)"),
            "note: {}",
            outcome.note
        );
    }

    #[test]
    fn wasm_backend_promise_reactions_run_after_synchronous_code_in_registration_order() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let settle;
            let promise = new Promise(function (resolve) {
                settle = resolve;
            });
            promise.then(function (value) { print("first:" + value); });
            promise.then(function (value) { print("second:" + value); });
            print("sync");
            settle(4);
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("pending Promise reactions should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync".to_string(),
                "first:4".to_string(),
                "second:4".to_string()
            ]
        );
    }

    #[test]
    fn wasm_backend_async_function_returns_promise_before_its_reaction_runs() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function immediate() { return 7; }
            let returned = immediate();
            print(returned instanceof Promise);
            returned.then(function (value) { print("fulfilled:" + value); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async function should return a Promise: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "true".to_string(),
                "sync".to_string(),
                "fulfilled:7".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_function_converts_synchronous_throw_to_rejection() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function thrown() { throw "sync"; }
            thrown().then(undefined, function (reason) { print("rejected:" + reason); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async throw should reject its result Promise: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["sync".to_string(), "rejected:sync".to_string()]
        );
    }

    #[test]
    fn wasm_backend_async_function_expression_resumes_lexical_state() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let expression = async function (value) {
                let retained = value;
                await Promise.resolve();
                return retained + 1;
            };
            let returned = expression(41);
            print(returned instanceof Promise);
            returned.then(function (value) { print("fulfilled:" + value); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async function expression should resume: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "true".to_string(),
                "sync".to_string(),
                "fulfilled:42".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_object_method_preserves_receiver_and_is_not_constructable() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let holder = {
                marker: 40,
                async method(delta) {
                    await Promise.resolve();
                    return this.marker + delta;
                }
            };
            print(holder.method.name);
            let returned = holder.method(2);
            print(returned instanceof Promise);
            try {
                new holder.method(1);
            } catch (error) {
                print(error instanceof TypeError);
            }
            returned.then(function (value) { print("fulfilled:" + value); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async object method should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "method".to_string(),
                "true".to_string(),
                "true".to_string(),
                "sync".to_string(),
                "fulfilled:42".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_object_method_rejects_later_parameter_tdz_read() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let holder = { async method(value = later, later) {} };
            holder.method().then(
                undefined,
                function (error) { print(error instanceof ReferenceError); }
            );
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async parameter TDZ should reject: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["sync".to_string(), "true".to_string()]
        );
    }

    #[test]
    fn wasm_backend_async_class_methods_preserve_receiver_super_and_method_semantics() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            class Base {
                method() { return this.marker; }
                static staticMethod() { return this.marker; }
            }
            class Derived extends Base {
                async method(delta) {
                    await Promise.resolve();
                    return super.method() + delta;
                }
                static async staticMethod(delta) {
                    await Promise.resolve();
                    return super.staticMethod() + delta;
                }
            }
            const instance = new Derived();
            instance.marker = 40;
            Derived.marker = 41;

            print(instance.method.name);
            print(Derived.staticMethod.name);
            const instanceResult = instance.method(2);
            const staticResult = Derived.staticMethod(2);
            print(instanceResult instanceof Promise);
            print(staticResult instanceof Promise);
            try { new instance.method(); } catch (error) {
                print(error instanceof TypeError);
            }
            try { new Derived.staticMethod(); } catch (error) {
                print(error instanceof TypeError);
            }
            instanceResult.then(function (value) { print("instance:" + value); });
            staticResult.then(function (value) { print("static:" + value); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async class methods should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "method".to_string(),
                "staticMethod".to_string(),
                "true".to_string(),
                "true".to_string(),
                "true".to_string(),
                "true".to_string(),
                "sync".to_string(),
                "instance:42".to_string(),
                "static:43".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_object_and_class_methods_use_real_activations() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            function delegate(prefix) {
                return {
                    [Symbol.asyncIterator]: function () { return this; },
                    next: function () {
                        return { value: prefix + ":open", done: false };
                    },
                    return: function (value) {
                        return { value: prefix + ":" + value, done: true };
                    }
                };
            }
            const object = {
                marker: "object",
                async *method() { yield* delegate(this.marker); }
            };
            class Example {
                constructor() { this.marker = "instance"; }
                async *method() { yield* delegate(this.marker); }
                static async *staticMethod() { yield* delegate(this.marker); }
            }
            Example.marker = "static";

            function observe(label, iterator) {
                return iterator.next().then(function (result) {
                    print(label + ":" + result.value + ":" + result.done);
                    return iterator.return("closed");
                }).then(function (result) {
                    print(label + ":" + result.value + ":" + result.done);
                });
            }

            const objectIterator = object.method();
            const instance = new Example();
            const instanceIterator = instance.method();
            const staticIterator = Example.staticMethod();
            print(Object.getPrototypeOf(objectIterator) === object.method.prototype);
            print(Object.getPrototypeOf(instanceIterator) === instance.method.prototype);
            print(Object.getPrototypeOf(staticIterator) === Example.staticMethod.prototype);
            observe("object", objectIterator)
                .then(function () { return observe("instance", instanceIterator); })
                .then(function () { return observe("static", staticIterator); });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator methods should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "true".to_string(),
                "true".to_string(),
                "true".to_string(),
                "object:object:open:false".to_string(),
                "object:object:closed:true".to_string(),
                "instance:instance:open:false".to_string(),
                "instance:instance:closed:true".to_string(),
                "static:static:open:false".to_string(),
                "static:static:closed:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_arrows_preserve_lexical_captures_and_are_not_constructable() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            function make(prefix) {
                const expression = async delta =>
                    this.marker + arguments[0] + delta
                        + (new.target === undefined ? 1 : 100);
                const block = async delta => {
                    await Promise.resolve();
                    return this.marker + arguments[0] + delta
                        + (new.target === undefined ? 1 : 100);
                };
                return [expression, block];
            }
            const holder = { marker: 40 };
            const arrows = make.call(holder, 1);
            const expression = arrows[0];
            const block = arrows[1];
            print(expression.name);
            print(block.name);
            const expressionResult = expression(0);
            const blockResult = block(2);
            print(expressionResult instanceof Promise);
            print(blockResult instanceof Promise);
            try { new expression(); } catch (error) {
                print(error instanceof TypeError);
            }
            try { new block(); } catch (error) {
                print(error instanceof TypeError);
            }
            expressionResult.then(function (value) { print("expression:" + value); });
            blockResult.then(function (value) { print("block:" + value); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async arrows should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "expression".to_string(),
                "block".to_string(),
                "true".to_string(),
                "true".to_string(),
                "true".to_string(),
                "true".to_string(),
                "sync".to_string(),
                "expression:42".to_string(),
                "block:44".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_for_await_array_awaits_values_and_preserves_iteration_bindings() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function collect() {
                const readers = [];
                for await (const value of [Promise.resolve(1), 2]) {
                    readers.push(function () { return value; });
                }
                return readers[0]() + readers[1]();
            }
            const result = collect();
            print(result instanceof Promise);
            result.then(function (value) { print("sum:" + value); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("array-backed for-await-of should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["true".to_string(), "sync".to_string(), "sum:3".to_string(),]
        );
    }

    #[test]
    fn wasm_backend_for_await_array_propagates_break_and_return() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function breakAfterOne() {
                let count = 0;
                for await (const value of [1, 2]) {
                    count += value;
                    break;
                }
                return count;
            }
            async function returnFirst() {
                for await (const value of [3, 4]) {
                    return value;
                }
            }
            breakAfterOne().then(function (value) { print("break:" + value); });
            returnFirst().then(function (value) { print("return:" + value); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("for-await-of completions should propagate: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync".to_string(),
                "break:1".to_string(),
                "return:3".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_for_await_array_rejections_enter_async_catch() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function catchesRejection() {
                try {
                    for await (const value of [Promise.reject("boom")]) {
                        print("unreachable:" + value);
                    }
                } catch (error) {
                    return "caught:" + error;
                }
                return "unreachable";
            }
            catchesRejection().then(function (value) { print(value); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("for-await-of rejection should be catchable: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["sync".to_string(), "caught:boom".to_string()]
        );
    }

    #[test]
    fn wasm_backend_for_await_array_uses_observable_sync_iterator_protocol() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let calls = "";
            let source = [1, 2, 3];
            Array.prototype[Symbol.iterator] = function () {
                "use strict";
                calls += this === source ? "i" : "w";
                let index = 0;
                let iterator = {
                    get next() {
                        calls += "g";
                        return function () {
                            calls += "n";
                            index += 1;
                            return index <= 3
                                ? { value: Promise.resolve(index), done: false }
                                : { value: undefined, done: true };
                        };
                    },
                    return: function () {
                        calls += this === iterator ? "r" : "w";
                        return { done: true };
                    }
                };
                return iterator;
            };
            async function collect() {
                let sum = 0;
                for await (const value of source) {
                    calls += value;
                    sum += value;
                    if (sum === 3) break;
                }
                return sum;
            }
            collect().then(function (result) { print(result + ":" + calls); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("array iterator protocol should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["sync".to_string(), "3:ign1n2r".to_string()]
        );
    }

    #[test]
    fn wasm_backend_for_await_array_caches_array_iterator_prototype_next() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let calls = "";
            let source = [Promise.resolve(1), 2];
            let iteratorPrototype = Object.getPrototypeOf(source[Symbol.iterator]());
            let originalNext = iteratorPrototype.next;
            Object.defineProperty(iteratorPrototype, "next", {
                configurable: true,
                get: function () {
                    calls += "g";
                    return function () {
                        calls += "n";
                        return originalNext.call(this);
                    };
                }
            });
            async function collect() {
                let sum = 0;
                for await (const value of source) {
                    calls += value;
                    sum += value;
                }
                return sum;
            }
            collect().then(function (result) { print(result + ":" + calls); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("array iterator next lookup should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["sync".to_string(), "3:gn1n2n".to_string()]
        );
    }

    #[test]
    fn wasm_backend_for_await_string_iterates_unicode_code_points() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function collect() {
                let summaries = [];
                for await (const value of "a\uD834\uDF06b\uD834\uDF06\uDF06\uD834") {
                    summaries.push(
                        value.length + ":" + value.charCodeAt(0) + ":" + value.charCodeAt(1)
                    );
                }
                return summaries.join("|");
            }
            collect().then(function (summary) { print(summary); });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("string for-await-of should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["1:97:NaN|2:55348:57094|1:98:NaN|2:55348:57094|1:57094:NaN|1:55348:NaN".to_string()]
        );
    }

    #[test]
    fn wasm_backend_for_await_string_awaits_sync_iterator_close_value() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let events = [];
            let stringIteratorPrototype = Object.getPrototypeOf(""[Symbol.iterator]());
            stringIteratorPrototype.return = function () {
                events.push("return:" + arguments.length);
                return {
                    value: Promise.resolve().then(function () {
                        events.push("close-value");
                    }),
                    done: true
                };
            };
            async function consume() {
                for await (const value of "ab") {
                    events.push("body:" + value);
                    break;
                }
                events.push("after");
            }
            consume().then(function () {
                events.push("settled");
                print(events.join(":"));
            });
            events.push("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("string for-await-of close should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["sync:body:a:return:0:close-value:after:settled".to_string()]
        );
    }

    #[test]
    fn wasm_backend_for_await_string_prefers_async_iterator_with_primitive_receiver() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let calls = "";
            let iterator = {
                index: 0,
                next: function () {
                    calls += "n";
                    this.index = this.index + 1;
                    return Promise.resolve(this.index === 1
                        ? { value: "async", done: false }
                        : { value: undefined, done: true });
                }
            };
            String.prototype[Symbol.asyncIterator] = function () {
                "use strict";
                calls += this === "source" ? "a" : "w";
                return iterator;
            };
            String.prototype[Symbol.iterator] = function () {
                calls += "s";
                throw new Error("sync iterator must not be selected");
            };
            async function collect() {
                let result = "";
                for await (const value of "source") result += value;
                return result;
            }
            collect().then(function (result) { print(result + ":" + calls); });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("string async iterator preference should execute: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["async:ann".to_string()]
        );
    }

    #[test]
    fn wasm_backend_for_await_sync_iterator_caches_next_and_awaits_values() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let nextGets = 0;
            let nextCalls = 0;
            let iterator = {};
            Object.defineProperty(iterator, "next", {
                get: function () {
                    nextGets += 1;
                    return function () {
                        nextCalls += 1;
                        if (nextCalls === 1) return { value: Promise.resolve(4), done: false };
                        if (nextCalls === 2) return { value: 5, done: false };
                        return { value: undefined, done: true };
                    };
                }
            });
            let iterable = {};
            iterable[Symbol.iterator] = function () { return iterator; };
            async function collect(iterable) {
                let total = 0;
                for await (const value of iterable) total += value;
                return total;
            }
            collect(iterable).then(function (value) {
                print(value + ":" + nextGets + ":" + nextCalls);
            });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("sync iterator for-await-of should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["sync".to_string(), "9:1:3".to_string()]
        );
    }

    #[test]
    fn wasm_backend_for_await_prefers_async_iterator_and_awaits_next_results() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let calls = "";
            let iterator = {
                index: 0,
                next: function () {
                    calls += "n";
                    this.index = this.index + 1;
                    return Promise.resolve(this.index === 1
                        ? { value: 9, done: false }
                        : {
                            get value() { throw new Error("done result value must not be read"); },
                            done: true
                        });
                }
            };
            let iterable = {};
            iterable[Symbol.asyncIterator] = function () {
                calls += "a";
                return iterator;
            };
            iterable[Symbol.iterator] = function () {
                calls += "s";
                throw new Error("sync iterator must not be selected");
            };
            async function collect() {
                let total = 0;
                for await (const value of iterable) total += value;
                return total;
            }
            collect().then(function (total) { print(total + ":" + calls); });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async iterator acquisition should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["9:ann".to_string()]
        );
    }

    #[test]
    fn wasm_backend_for_await_dispatches_native_synchronous_next_throws_before_await() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let synchronousError = {};
            let rejectionError = {};
            let events = [];
            function asyncIterable(next) {
                let iterator = { next: next };
                let iterable = {};
                iterable[Symbol.asyncIterator] = function () { return iterator; };
                return iterable;
            }
            async function consumeSynchronousThrow() {
                try {
                    for await (const value of asyncIterable(function () {
                        throw synchronousError;
                    })) {}
                } catch (error) {
                    events.push(error === synchronousError ? "sync-caught" : "sync-wrong");
                }
            }
            async function consumeRejectedNext() {
                try {
                    for await (const value of asyncIterable(function () {
                        return Promise.reject(rejectionError);
                    })) {}
                } catch (error) {
                    events.push(error === rejectionError ? "reject-caught" : "reject-wrong");
                }
            }
            Promise.resolve().then(function () { events.push("queued"); });
            let synchronousResult = consumeSynchronousThrow();
            events.push("after-call");
            synchronousResult.then(function () {
                events.push("sync-settled");
                consumeRejectedNext().then(function () {
                    events.push("reject-settled");
                    print(events.join(":"));
                });
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("native async iterator next failures should execute: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync-caught:after-call:queued:sync-settled:reject-caught:reject-settled"
                    .to_string()
            ]
        );
    }

    #[test]
    fn wasm_backend_for_await_falls_back_only_for_nullish_async_iterator_methods() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            function syncIterable(asyncMethod) {
                let iterable = {};
                iterable[Symbol.asyncIterator] = asyncMethod;
                iterable[Symbol.iterator] = function () {
                    let done = false;
                    return {
                        next: function () {
                            if (done) return { done: true };
                            done = true;
                            return { value: Promise.resolve(3), done: false };
                        }
                    };
                };
                return iterable;
            }
            async function first(iterable) {
                for await (const value of iterable) return value;
            }
            async function rejectsNonCallable() {
                try {
                    for await (const value of syncIterable(1)) {}
                } catch (error) {
                    return error instanceof TypeError;
                }
                return false;
            }
            first(syncIterable(null)).then(function (fallbackValue) {
                rejectsNonCallable().then(function (rejected) {
                    print(fallbackValue + ":" + rejected);
                });
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async iterator fallback should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["3:true".to_string()]
        );
    }

    #[test]
    fn wasm_backend_for_await_awaits_and_validates_async_iterator_close_results() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            function asyncIterable(closeResult) {
                let closeCalls = 0;
                let iterator = {
                    next: function () {
                        return Promise.resolve({ value: 1, done: false });
                    },
                    return: function () {
                        closeCalls += 1;
                        return Promise.resolve(closeResult);
                    }
                };
                let iterable = { closeCalls: function () { return closeCalls; } };
                iterable[Symbol.asyncIterator] = function () { return iterator; };
                return iterable;
            }
            async function closes(iterable) {
                for await (const value of iterable) break;
                return iterable.closeCalls();
            }
            async function rejectsPrimitiveClose(iterable) {
                try {
                    for await (const value of iterable) break;
                } catch (error) {
                    return error instanceof TypeError && iterable.closeCalls() === 1;
                }
                return false;
            }
            let valid = asyncIterable({});
            let invalid = asyncIterable(1);
            closes(valid).then(function (closeCalls) {
                rejectsPrimitiveClose(invalid).then(function (rejected) {
                    print(closeCalls + ":" + rejected);
                });
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async iterator close should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["1:true".to_string()]
        );
    }

    #[test]
    fn wasm_backend_for_await_applies_async_close_get_method_abrupt_precedence() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let bodyError = {};
            let closeError = {};
            let closeGetterCalls = 0;
            function iterableWithThrowingReturn() {
                let iterator = {
                    next: function () { return { value: 1, done: false }; },
                    get return() {
                        closeGetterCalls += 1;
                        throw closeError;
                    }
                };
                let iterable = {};
                iterable[Symbol.asyncIterator] = function () { return iterator; };
                return iterable;
            }
            function iterableWithNonCallableReturn() {
                let iterator = {
                    next: function () { return { value: 1, done: false }; },
                    return: 1
                };
                let iterable = {};
                iterable[Symbol.asyncIterator] = function () { return iterator; };
                return iterable;
            }
            async function closeFailureWinsBreak() {
                try {
                    for await (const value of iterableWithThrowingReturn()) break;
                } catch (error) {
                    return error === closeError;
                }
                return false;
            }
            async function bodyFailureWinsClose() {
                try {
                    for await (const value of iterableWithThrowingReturn()) throw bodyError;
                } catch (error) {
                    return error === bodyError;
                }
                return false;
            }
            async function nonCallableCloseFailureWinsBreak() {
                try {
                    for await (const value of iterableWithNonCallableReturn()) break;
                } catch (error) {
                    return error instanceof TypeError;
                }
                return false;
            }
            async function bodyFailureWinsNonCallableClose() {
                try {
                    for await (const value of iterableWithNonCallableReturn()) throw bodyError;
                } catch (error) {
                    return error === bodyError;
                }
                return false;
            }
            closeFailureWinsBreak().then(function (closeWins) {
                bodyFailureWinsClose().then(function (bodyWins) {
                    nonCallableCloseFailureWinsBreak().then(function (nonCallableCloseWins) {
                        bodyFailureWinsNonCallableClose().then(function (nonCallableBodyWins) {
                            print(
                                closeWins + ":" +
                                bodyWins + ":" +
                                closeGetterCalls + ":" +
                                nonCallableCloseWins + ":" +
                                nonCallableBodyWins
                            );
                        });
                    });
                });
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async iterator GetMethod close should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["true:true:2:true:true".to_string()]
        );
    }

    #[test]
    fn wasm_backend_for_await_sync_iterator_closes_on_break() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let closeCalls = 0;
            let closeArguments = -1;
            function iterableWith(value) {
                let iterator = {
                    next: function () { return { value: value, done: false }; },
                    return: function () {
                        closeCalls += 1;
                        closeArguments = arguments.length;
                        return { value: undefined, done: true };
                    }
                };
                let iterable = {};
                iterable[Symbol.iterator] = function () { return iterator; };
                return iterable;
            }
            async function breakAfterOne() {
                for await (const value of iterableWith(6)) break;
                return "break";
            }
            breakAfterOne().then(function (value) {
                print(value + ":" + closeCalls + ":" + closeArguments);
            });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("sync iterator close should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["sync".to_string(), "break:1:0".to_string()]
        );
    }

    #[test]
    fn wasm_backend_for_await_sync_iterator_closes_on_return() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let closeCalls = 0;
            let closeArguments = -1;
            let iterator = {
                next: function () { return { value: 7, done: false }; },
                return: function () {
                    closeCalls += 1;
                    closeArguments = arguments.length;
                    return { value: undefined, done: true };
                }
            };
            let iterable = {};
            iterable[Symbol.iterator] = function () { return iterator; };
            async function returnFirst(iterable) {
                for await (const value of iterable) return value;
            }
            returnFirst(iterable).then(function (value) {
                print(value + ":" + closeCalls + ":" + closeArguments);
            });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("sync iterator return close should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["sync".to_string(), "7:1:0".to_string()]
        );
    }

    #[test]
    fn wasm_backend_for_await_sync_iterator_preserves_throw_close_precedence() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let bodyError = {};
            let closeError = {};
            function iterableWithReturn(returnDescriptor) {
                let iterator = {
                    next: function () { return { value: 1, done: false }; }
                };
                Object.defineProperty(iterator, "return", returnDescriptor);
                let iterable = {};
                iterable[Symbol.iterator] = function () { return iterator; };
                return iterable;
            }
            async function thrownBodyWins() {
                try {
                    for await (const value of iterableWithReturn({
                        get: function () { throw closeError; }
                    })) throw bodyError;
                } catch (error) {
                    return error === bodyError;
                }
            }
            async function closeFailureWinsBreak() {
                try {
                    for await (const value of iterableWithReturn({
                        value: function () { throw closeError; }
                    })) break;
                } catch (error) {
                    return error === closeError;
                }
                return false;
            }
            thrownBodyWins().then(function (bodyWins) {
                closeFailureWinsBreak().then(function (closeWins) {
                    print(bodyWins && closeWins);
                });
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("sync iterator close precedence should execute: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["true".to_string()]
        );
    }

    #[test]
    fn wasm_backend_for_await_sync_iterator_rejects_abrupt_and_non_object_results() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let nextError = {};
            let doneError = {};
            let valueError = {};
            let closeCalls = 0;
            function iterableWithNext(next) {
                let iterable = {};
                iterable[Symbol.iterator] = function () {
                    return {
                        next: next,
                        return: function () {
                            closeCalls += 1;
                            return {};
                        }
                    };
                };
                return iterable;
            }
            async function catches(iterable, expected, expectsTypeError) {
                try {
                    for await (const value of iterable) {}
                } catch (error) {
                    return expectsTypeError ? error instanceof TypeError : error === expected;
                }
                return false;
            }
            let settled = 0;
            let passed = 0;
            function record(result) {
                if (result) passed += 1;
                settled += 1;
                if (settled === 4) print(passed === 4 && closeCalls === 0);
            }
            catches(iterableWithNext(function () { throw nextError; }), nextError, false).then(record);
            catches(iterableWithNext(function () { return 1; }), undefined, true).then(record);
            catches(iterableWithNext(function () {
                return {
                    get done() { throw doneError; }
                };
            }), doneError, false).then(record);
            catches(iterableWithNext(function () {
                return {
                    done: false,
                    get value() { throw valueError; }
                };
            }), valueError, false).then(record);
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("sync iterator abrupt results should reject: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["true".to_string()]
        );
    }

    #[test]
    fn wasm_backend_async_finalizers_override_pending_completions() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let rejects = async function () {
                try { return "early-return"; }
                finally {
                    await Promise.reject("override-reject");
                    print("unreachable-reject");
                }
            };
            let fulfills = async function () {
                try { await Promise.reject("early-reject"); }
                finally {
                    return await Promise.resolve("override-return");
                    print("unreachable-return");
                }
            };
            let catches = async function () {
                try { await Promise.reject("caught-reject"); }
                catch (error) {
                    await Promise.resolve();
                    return error;
                } finally {
                    await Promise.resolve();
                }
            };
            rejects().then(undefined, function (value) { print("rejected:" + value); });
            fulfills().then(function (value) { print("fulfilled:" + value); });
            catches().then(function (value) { print("caught:" + value); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async finalizers should settle promises: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync".to_string(),
                "rejected:override-reject".to_string(),
                "fulfilled:override-return".to_string(),
                "caught:caught-reject".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_await_resumes_lexical_state_and_propagates_rejection() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function retained() {
                let value = 40;
                let awaited;
                awaited = await Promise.resolve(2);
                return value + awaited;
            }
            async function rejected() {
                await Promise.reject("bad");
                return "unreachable";
            }
            retained().then(function (value) { print("fulfilled:" + value); });
            rejected().then(undefined, function (reason) { print("rejected:" + reason); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("async await should resume through Promise jobs: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync".to_string(),
                "fulfilled:42".to_string(),
                "rejected:bad".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_await_resumes_once_and_drains_queued_next_requests() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let prefixRuns = 0;
            let suffixRuns = 0;
            let release;
            let blocking = new Promise(function (resolve) { release = resolve; });
            async function* stream() {
                prefixRuns += 1;
                let value;
                value = await blocking;
                suffixRuns += 1;
                return value + 1;
            }
            let iterator = stream();
            let first = iterator.next();
            let second = iterator.next();
            first.then(function (result) {
                print("first:" + result.value + ":" + result.done + ":" + prefixRuns + ":" + suffixRuns);
            });
            second.then(function (result) {
                print("second:" + result.value + ":" + result.done);
            });
            print("sync:" + prefixRuns + ":" + suffixRuns);
            release(41);
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator Await should resume: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync:1:0".to_string(),
                "first:42:true:1:1".to_string(),
                "second:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_yields_settle_queued_requests_in_fifo_order() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let settled = 0;
            let prefixRuns = 0;
            async function* stream() {
                prefixRuns += 1;
                yield "first";
                yield "second";
            }
            let iterator = stream();
            let first = iterator.next();
            let second = iterator.next();
            let third = iterator.next();
            third.then(function (result) {
                settled += 1;
                print("third:" + settled + ":" + result.value + ":" + result.done + ":" + prefixRuns);
            });
            second.then(function (result) {
                settled += 1;
                print("second:" + settled + ":" + result.value + ":" + result.done);
            });
            first.then(function (result) {
                settled += 1;
                print("first:" + settled + ":" + result.value + ":" + result.done);
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("queued async-generator yields should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "first:1:first:false".to_string(),
                "second:2:second:false".to_string(),
                "third:3:undefined:true:1".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_initializes_parameters_before_selecting_instance_prototype() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let bodyStarted = false;
            let stream = async function* (value = (stream.prototype = null)) {
                bodyStarted = true;
            };
            let oldPrototype = stream.prototype;
            let iterator = stream();
            print((Object.getPrototypeOf(iterator) !== oldPrototype) + ":" + bodyStarted);
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("async-generator parameter initialization should run: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["true:false".to_string()]
        );
    }

    #[test]
    fn wasm_backend_async_generator_executes_no_suspension_conditional_branches() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function* choose(flag) {
                let value = 0;
                if (flag) {
                    value = 1;
                    if (false) value = 2;
                } else {
                    value = 3;
                }
                print(value);
            }
            choose(true).next();
            choose(false).next();
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("async-generator ordinary conditionals should run: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["1".to_string(), "3".to_string()]
        );
    }

    #[test]
    fn wasm_backend_async_generator_await_rejects_active_request_and_completes_queue() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let reason = {};
            async function* stream() {
                await Promise.reject(reason);
                return "unreachable";
            }
            let iterator = stream();
            iterator.next().then(undefined, function (error) {
                print("rejected:" + (error === reason));
            });
            iterator.next().then(function (result) {
                print("completed:" + result.value + ":" + result.done);
            });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("rejected async-generator Await should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync".to_string(),
                "rejected:true".to_string(),
                "completed:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_return_await_assimilates_thenable_once() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let getterCalls = 0;
            let thenable = {};
            Object.defineProperty(thenable, "then", {
                get: function () {
                    getterCalls += 1;
                    return function (resolve, reject) {
                        resolve(8);
                        reject(9);
                    };
                }
            });
            async function* stream() { return await thenable; }
            stream().next().then(function (result) {
                print("result:" + result.value + ":" + result.done + ":" + getterCalls);
            });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator return Await should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["sync".to_string(), "result:8:true:1".to_string()]
        );
    }

    #[test]
    fn wasm_backend_async_generator_yield_statements_resolve_before_completion() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function* withoutValue() { yield; }
            async function* withValue() { yield 1; }
            let first = withoutValue();
            first.next().then(function (result) {
                print("without:first:" + result.value + ":" + result.done);
            });
            first.next().then(function (result) {
                print("without:second:" + result.value + ":" + result.done);
            });
            let second = withValue();
            second.next().then(function (result) {
                print("with:first:" + result.value + ":" + result.done);
            });
            second.next().then(function (result) {
                print("with:second:" + result.value + ":" + result.done);
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator yield statements should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "without:first:undefined:false".to_string(),
                "without:second:undefined:true".to_string(),
                "with:first:1:false".to_string(),
                "with:second:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_yield_assimilates_resolve_first_thenables_once() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let getterCalls = 0;
            let thenCalls = 0;
            let late = {};
            let thenable = {};
            Object.defineProperty(thenable, "then", {
                get: function () {
                    getterCalls += 1;
                    return function (resolve, reject) {
                        thenCalls += 1;
                        resolve("first");
                        reject(late);
                        resolve("last");
                        throw late;
                    };
                }
            });
            async function* stream() { yield thenable; }
            stream().next().then(function (result) {
                print("result:" + result.value + ":" + result.done + ":" + getterCalls + ":" + thenCalls);
            }, function () {
                print("unexpected rejection");
            });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("resolve-first thenable yield should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["sync".to_string(), "result:first:false:1:1".to_string(),]
        );
    }

    #[test]
    fn wasm_backend_async_generator_yield_assimilates_reject_first_thenables_once() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let reason = {};
            let late = {};
            let thenable = {
                then: function (resolve, reject) {
                    reject(reason);
                    resolve("late");
                    reject(late);
                    throw late;
                }
            };
            let iterator = (async function*() { yield thenable; }());
            iterator.next().then(function () {
                print("unexpected fulfillment");
            }, function (rejected) {
                print("rejected:" + (rejected === reason));
                iterator.next().then(function (result) {
                    print("closed:" + result.value + ":" + result.done);
                });
            });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("reject-first thenable yield should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync".to_string(),
                "rejected:true".to_string(),
                "closed:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_yield_await_stages_the_resolved_value() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let thenable = {
                then(resolve) {
                    resolve("ready");
                }
            };
            async function* stream(value) { yield await value; }
            let iterator = stream(thenable);
            iterator.next().then(function (result) {
                print("first:" + result.value + ":" + result.done);
            });
            iterator.next().then(function (result) {
                print("second:" + result.value + ":" + result.done);
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator yield-await should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "first:ready:false".to_string(),
                "second:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_resumes_alternating_awaits_and_yields() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function* stream(first, second) {
                await first;
                yield "first";
                await second;
                yield "second";
                return "done";
            }
            let second = {
                then(resolve) {
                    resolve("second");
                }
            };
            let iterator = stream(Promise.resolve("first"), second);
            iterator.next()
                .then(function (result) {
                    print(result.value + ":" + result.done);
                    return iterator.next();
                })
                .then(function (result) {
                    print(result.value + ":" + result.done);
                    return iterator.next();
                })
                .then(function (result) {
                    print(result.value + ":" + result.done);
                });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("async-generator alternating suspensions should execute: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "first:false".to_string(),
                "second:false".to_string(),
                "done:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_yield_star_uses_null_async_method_sync_fallback() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let events = [];
            let delegate = {
                get [Symbol.asyncIterator]() {
                    events.push("get async");
                    return null;
                },
                get [Symbol.iterator]() {
                    events.push("get sync");
                    return function () {
                        events.push("call sync:" + (this === delegate) + ":" + arguments.length);
                        return {
                            get next() {
                                events.push("get next");
                                return function () {
                                    events.push("call next");
                                    return { value: "value", done: false };
                                };
                            }
                        };
                    };
                }
            };
            async function* outer() { yield* delegate; }
            outer().next().then(function (result) {
                events.push("result:" + result.value + ":" + result.done);
                print(events.join("|"));
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator sync fallback should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "get async|get sync|call sync:true:0|get next|call next|result:value:false"
                    .to_string()
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_yield_star_rejects_sync_iterator_acquisition_failures() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let getReason = {};
            let callReason = {};
            let syncGetterCalls = 0;
            let getterFailure = {
                get [Symbol.asyncIterator]() { throw getReason; },
                get [Symbol.iterator]() {
                    syncGetterCalls += 1;
                    return function () { return {}; };
                }
            };
            let callFailure = {
                [Symbol.iterator]: function () { throw callReason; }
            };
            let resultFailure = {
                [Symbol.iterator]: function () { return 1; }
            };
            function rejectionFrom(delegate) {
                return (async function* () { yield* delegate; })().next();
            }
            rejectionFrom(getterFailure).then(undefined, function (error) {
                print("get:" + (error === getReason) + ":" + syncGetterCalls);
                return rejectionFrom(callFailure);
            }).then(undefined, function (error) {
                print("call:" + (error === callReason));
                return rejectionFrom(resultFailure);
            }).then(undefined, function (error) {
                print("result:" + (error.constructor === TypeError));
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("async-generator sync acquisition failures should reject: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "get:true:0".to_string(),
                "call:true".to_string(),
                "result:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_yield_star_delegates_each_result() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function* inner() { yield 1; }
            async function* outer() { yield* inner(); }
            let iterator = outer();
            iterator.next().then(function (result) {
                print("first:" + result.value + ":" + result.done);
            });
            iterator.next().then(function (result) {
                print("second:" + result.value + ":" + result.done);
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator delegation should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "first:1:false".to_string(),
                "second:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_yield_star_forwards_next_and_assigns_completion() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let log = [];
            let calls = 0;
            let delegate = {
                get [Symbol.asyncIterator]() {
                    log.push("get async iterator");
                    return function () {
                        let passedArguments = [...arguments];
                        log.push("call async iterator:" + (this === delegate) + ":" + passedArguments.length);
                        return this;
                    };
                },
                get next() {
                    log.push("get next:" + (this === delegate));
                    return function (value) {
                        let passedArguments = [...arguments];
                        calls += 1;
                        log.push("call next:" + (this === delegate) + ":" + passedArguments.length + ":" + passedArguments[0]);
                        let call = calls;
                        return {
                            get then() {
                                log.push("get then:" + call);
                                return function (resolve, reject) {
                                    let passedArguments = [...arguments];
                                    log.push("call then:" + call + ":" + (typeof passedArguments[0]) + ":" + (typeof passedArguments[1]));
                                    resolve({
                                        get done() {
                                            log.push("get done:" + call);
                                            return call !== 1;
                                        },
                                        get value() {
                                            log.push("get value:" + call);
                                            return call === 1 ? "first" : "complete";
                                        }
                                    });
                                };
                            }
                        };
                    };
                }
            };
            async function* outer() {
                log.push("before yield star");
                var completion = yield* delegate;
                log.push("after yield star:" + completion);
                return completion;
            }
            let iterator = outer();
            iterator.next("ignored").then(result => {
                print("first:" + result.value + ":" + result.done + "|" + log.join(","));
                return iterator.next("forwarded");
            }).then(result => {
                print("second:" + result.value + ":" + result.done + "|" + log.join(","));
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator delegated next should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "first:first:false|before yield star,get async iterator,call async iterator:true:0,get next:true,call next:true:1:undefined,get then:1,call then:1:function:function,get done:1,get value:1".to_string(),
                "second:complete:true|before yield star,get async iterator,call async iterator:true:0,get next:true,call next:true:1:undefined,get then:1,call then:1:function:function,get done:1,get value:1,call next:true:1:forwarded,get then:2,call then:2:function:function,get done:2,get value:2,after yield star:complete".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_yield_star_forwards_return_to_sync_iterators() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let returnCalls = 0;
            let delegate = {
                [Symbol.iterator]: function () { return this; },
                next: function () { return { value: "first", done: false }; },
                return: function (value) {
                    returnCalls += 1;
                    print("delegate:return:" + value);
                    return returnCalls === 1
                        ? { value: "continue", done: false }
                        : { value: "closed", done: true };
                }
            };
            async function* outer() { yield* delegate; }
            let iterator = outer();
            iterator.next().then(function (result) {
                print("first:" + result.value + ":" + result.done);
                return iterator.return("one");
            }).then(function (result) {
                print("second:" + result.value + ":" + result.done);
                return iterator.return("two");
            }).then(function (result) {
                print("third:" + result.value + ":" + result.done);
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator delegated return should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "first:first:false".to_string(),
                "delegate:return:one".to_string(),
                "second:continue:false".to_string(),
                "delegate:return:two".to_string(),
                "third:closed:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_catches_abrupt_delegated_return_value() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            const token = {};
            const delegate = {
                [Symbol.asyncIterator]() { return this; },
                next() { return { value: "open", done: false }; },
                return() {
                    return {
                        done: false,
                        get value() { throw token; }
                    };
                }
            };
            async function* outer() {
                let caught;
                try {
                    yield* delegate;
                } catch (error) {
                    caught = error;
                }
                return caught;
            }
            const iterator = outer();
            iterator.next()
                .then(function () { return iterator.return(); })
                .then(function (result) {
                    print((result.value === token) + ":" + result.done);
                });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("async-generator delegated return catch should execute: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["true:true".to_string()]
        );
    }

    #[test]
    fn wasm_backend_async_generator_finally_preserves_return_across_yield() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function* values() {
                try {
                    yield "open";
                } finally {
                    yield "cleanup";
                }
            }
            const iterator = values();
            iterator.next()
                .then(function (result) {
                    print(result.value + ":" + result.done);
                    return iterator.return("closed");
                })
                .then(function (result) {
                    print(result.value + ":" + result.done);
                    return iterator.next();
                })
                .then(function (result) {
                    print(result.value + ":" + result.done);
                });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("async-generator finalizer should preserve return: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "open:false".to_string(),
                "cleanup:false".to_string(),
                "closed:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_finally_throw_overrides_return() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            var overrideReason = new Error("override");
            var values = async function*() {
                try {
                    yield "open";
                } finally {
                    throw overrideReason;
                    throw new Error("unreachable");
                }
            };
            const iterator = values();
            iterator.next()
                .then(function () {
                    return iterator.return("closed");
                })
                .then(function () {
                    print("fulfilled");
                }, function (reason) {
                    print("rejected:" + (reason === overrideReason));
                    print("message:" + reason.message);
                    return iterator.next();
                })
                .then(function (result) {
                    print("closed:" + result.value + ":" + result.done);
                });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("async-generator finalizer throw should execute: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "rejected:true".to_string(),
                "message:override".to_string(),
                "closed:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_catches_return_await_errors() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            const token = new Error("broken promise");
            let caught;
            const brokenPromise = Promise.resolve(42);
            Object.defineProperty(brokenPromise, "constructor", {
                get() {
                    throw token;
                }
            });
            const values = async function*() {
                try {
                    yield "open";
                } catch (error) {
                    caught = error;
                    return "caught";
                }
            };
            const iterator = values();
            iterator.next()
                .then(function () {
                    return iterator.return(brokenPromise);
                })
                .then(function (result) {
                    print("caught:" + (caught === token));
                    print("result:" + result.value + ":" + result.done);
                }, function (reason) {
                    print("rejected:" + (reason === token));
                });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("async-generator return-await error should execute: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["caught:true".to_string(), "result:caught:true".to_string()]
        );
    }

    #[test]
    fn wasm_backend_async_generator_catch_finally_resumes_each_clause() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            const token = {};
            async function* values() {
                try {
                    yield Promise.reject(token);
                } catch (error) {
                    yield error;
                } finally {
                    yield "cleanup";
                }
            }
            const iterator = values();
            iterator.next()
                .then(function (result) {
                    print((result.value === token) + ":" + result.done);
                    return iterator.next();
                })
                .then(function (result) {
                    print(result.value + ":" + result.done);
                    return iterator.next();
                })
                .then(function (result) {
                    print(result.value + ":" + result.done);
                });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator catch/finally should resume: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "true:false".to_string(),
                "cleanup:false".to_string(),
                "undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_yield_star_awaits_return_before_delegate_return_lookup() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let events = [];
            let delegate = {
                [Symbol.asyncIterator]: function () { return this; },
                next: function () { return { done: false }; },
                get return() {
                    events.push("get return");
                    return undefined;
                }
            };
            async function* outer() {
                events.push("start");
                yield* delegate;
            }
            Promise.resolve(0)
                .then(function () { events.push("tick 1"); })
                .then(function () { events.push("tick 2"); })
                .then(function () { events.push("tick 3"); })
                .then(function () { print(events.join("|")); });
            let iterator = outer();
            iterator.next();
            iterator.return({
                get then() {
                    events.push("get then");
                    return undefined;
                }
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator return resumption should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["start|tick 1|get then|tick 2|get return|get then|tick 3".to_string()]
        );
    }

    #[test]
    fn wasm_backend_async_generator_yield_awaits_return_before_next_promise_tick() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let events = [];
            async function* outer() {
                events.push("start");
                yield 123;
            }
            Promise.resolve(0)
                .then(function () { events.push("tick 1"); })
                .then(function () { events.push("tick 2"); })
                .then(function () { print(events.join("|")); });
            let iterator = outer();
            iterator.next();
            iterator.return({
                get then() {
                    events.push("get then");
                    return undefined;
                }
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator return resumption should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["start|tick 1|get then|tick 2".to_string()]
        );
    }

    #[test]
    fn wasm_backend_async_generator_yield_star_forwards_throw_to_sync_iterators() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let throwCalls = 0;
            let delegate = {
                [Symbol.iterator]: function () { return this; },
                next: function () { return { value: "first", done: false }; },
                throw: function (value) {
                    throwCalls += 1;
                    print("delegate:throw:" + value);
                    return throwCalls === 1
                        ? { value: "continue", done: false }
                        : { value: "complete", done: true };
                }
            };
            async function* outer() {
                var completion = yield* delegate;
                return "outer:" + completion;
            }
            let iterator = outer();
            iterator.next().then(function (result) {
                print("first:" + result.value + ":" + result.done);
                return iterator.throw("one");
            }).then(function (result) {
                print("second:" + result.value + ":" + result.done);
                return iterator.throw("two");
            }).then(function (result) {
                print("third:" + result.value + ":" + result.done);
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator delegated throw should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "first:first:false".to_string(),
                "delegate:throw:one".to_string(),
                "second:continue:false".to_string(),
                "delegate:throw:two".to_string(),
                "third:outer:complete:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_yield_star_closes_before_missing_throw_type_error() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let delegate = {
                [Symbol.asyncIterator]: function () { return this; },
                next: function () { return { value: "first", done: false }; },
                get throw() {
                    print("get throw");
                    return null;
                },
                get return() {
                    print("get return");
                    return function () {
                        print("call return");
                        return {
                            then: function (resolve) {
                                print("await return");
                                resolve({ value: "closed", done: true });
                            }
                        };
                    };
                }
            };
            async function* outer() { yield* delegate; }
            let iterator = outer();
            iterator.next().then(function () {
                return iterator.throw("reason");
            }).then(function () {
                print("unexpected fulfillment");
            }, function (error) {
                print("rejected:" + (error.constructor === TypeError));
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator delegated close should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "get throw".to_string(),
                "get return".to_string(),
                "call return".to_string(),
                "await return".to_string(),
                "rejected:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_rejected_yield_closes_the_generator() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let reason = {};
            async function* stream() {
                yield Promise.reject(reason);
                yield "unreachable";
            }
            let iterator = stream();
            iterator.next().then(function () {
                print("unexpected");
            }, function (rejected) {
                print("rejected:" + (rejected === reason));
                iterator.next().then(function (result) {
                    print("closed:" + result.value + ":" + result.done);
                });
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator rejected yield should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "rejected:true".to_string(),
                "closed:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_for_await_forwards_rejected_yields() {
        for source_kind in ["async", "sync"] {
            let lines = Arc::new(Mutex::new(Vec::new()));
            let source = if source_kind == "async" {
                r#"
                    let reason = {};
                    async function* source() { yield Promise.reject(reason); }
                    async function* stream() {
                        for await (let value of source()) { yield value; }
                    }
                    let iterator = stream();
                    iterator.next().then(function (result) {
                        print("unexpected:" + result.value + ":" + result.done);
                    }, function (rejected) {
                        print("rejected:" + (rejected === reason));
                        iterator.next().then(function (result) {
                            print("closed:" + result.value + ":" + result.done);
                        });
                    });
                "#
            } else {
                r#"
                    let reason = {};
                    let iterable = [Promise.reject(reason)];
                    async function* stream() {
                        for await (let value of iterable) { yield value; }
                    }
                    let iterator = stream();
                    iterator.next().then(function (result) {
                        print("unexpected:" + result.value + ":" + result.done);
                    }, function (rejected) {
                        print("rejected:" + (rejected === reason));
                        iterator.next().then(function (result) {
                            print("closed:" + result.value + ":" + result.done);
                        });
                    });
                "#
            };
            engine_with_captured_prints(Arc::clone(&lines))
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| {
                    panic!("async-generator for-await {source_kind} rejection should run: {err:?}")
                });
            assert_eq!(
                lines.lock().expect("capture mutex poisoned").as_slice(),
                &[
                    "rejected:true".to_string(),
                    "closed:undefined:true".to_string(),
                ],
                "{source_kind} iterator"
            );
        }
    }

    #[test]
    fn wasm_backend_async_generator_sync_yield_star_awaits_rejected_values() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let reason = {};
            let iterable = [Promise.reject(reason)];
            async function* stream() { yield* iterable; }
            let iterator = stream();
            iterator.next().then(function (result) {
                print("unexpected:" + result.value + ":" + result.done);
            }, function (rejected) {
                print("rejected:" + (rejected === reason));
                iterator.next().then(function (result) {
                    print("closed:" + result.value + ":" + result.done);
                });
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("async-generator sync yield-star rejection should run: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "rejected:true".to_string(),
                "closed:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_for_await_does_not_forward_next_values() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let received;
            async function* source() {
                received = yield 1;
                print("source:" + received);
            }
            async function* stream() {
                for await (let value of source()) { yield value; }
            }
            let iterator = stream();
            iterator.next().then(function (result) {
                print("first:" + result.value + ":" + result.done);
                return iterator.next(99);
            }).then(function (result) {
                print("second:" + result.value + ":" + result.done);
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator for-await next should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "first:1:false".to_string(),
                "source:undefined".to_string(),
                "second:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_array_spread_resumes_with_an_iterable() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function* stream() { yield [0, ...yield, 3]; }
            let iterator = stream();
            iterator.next().then(function () {
                return iterator.next("ab");
            }).then(function (result) {
                print(result.value.join(",") + ":" + result.done);
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator array spread should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["0,a,b,3:false".to_string()]
        );
    }

    #[test]
    fn wasm_backend_async_generator_object_spreads_resume_in_source_order() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function* stream() {
                yield { ...yield, y: 1, ...yield yield };
            }
            let iterator = stream();
            iterator.next();
            iterator.next({ x: 42 });
            iterator.next({ x: "ignored" });
            iterator.next({ y: 39 }).then(function (result) {
                print(result.value.x + ":" + result.value.y + ":" + result.done);
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("async-generator object spreads should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["42:39:false".to_string()]
        );
    }

    #[test]
    fn wasm_backend_async_generator_nested_yield_resumes_each_operand() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function* stream() { yield yield 1; }
            let iterator = stream();
            iterator.next().then(function (result) {
                print("first:" + result.value + ":" + result.done);
            });
            iterator.next().then(function (result) {
                print("second:" + result.value + ":" + result.done);
            });
            iterator.next().then(function (result) {
                print("third:" + result.value + ":" + result.done);
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("nested async-generator yields should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "first:1:false".to_string(),
                "second:undefined:false".to_string(),
                "third:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_newline_terminates_yield_operand() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            async function* stream() {
                yield
                1;
            }
            let iterator = stream();
            iterator.next().then(function (result) {
                print("first:" + result.value + ":" + result.done);
            });
            iterator.next().then(function (result) {
                print("second:" + result.value + ":" + result.done);
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("newline-terminated async-generator yield should run: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "first:undefined:false".to_string(),
                "second:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_return_closes_suspended_yield() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let resumed = false;
            async function* stream() {
                yield 1;
                resumed = true;
            }
            let iterator = stream();
            iterator.next().then(function () {
                iterator.return("sent-value").then(function (result) {
                    print("return:" + result.value + ":" + result.done + ":" + resumed);
                    iterator.next().then(function (closed) {
                        print("closed:" + closed.value + ":" + closed.done);
                    });
                });
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("return from suspended async-generator yield should run: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "return:sent-value:true:false".to_string(),
                "closed:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_return_awaits_value_from_suspended_yield() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let resumed = false;
            let release;
            let returned = new Promise(function (resolve) { release = resolve; });
            async function* stream() {
                yield 1;
                resumed = true;
            }
            let iterator = stream();
            iterator.next().then(function () {
                iterator.return(returned).then(function (result) {
                    print("return:" + result.value + ":" + result.done + ":" + resumed);
                    iterator.next().then(function (closed) {
                        print("closed:" + closed.value + ":" + closed.done);
                    });
                });
                release("unwrapped-value");
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("promised return from suspended async-generator yield should run: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "return:unwrapped-value:true:false".to_string(),
                "closed:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_throw_rejects_suspended_yield_by_identity() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let resumed = false;
            let reason = {};
            async function* stream() {
                yield 1;
                resumed = true;
            }
            let iterator = stream();
            iterator.next().then(function () {
                iterator.throw(reason).then(undefined, function (error) {
                    print("throw:" + (error === reason) + ":" + resumed);
                    iterator.next().then(function (closed) {
                        print("closed:" + closed.value + ":" + closed.done);
                    });
                });
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("throw into suspended async-generator yield should run: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "throw:true:false".to_string(),
                "closed:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_async_generator_throw_does_not_await_promise_reason() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let resumed = false;
            let reason = new Promise(function () {});
            async function* stream() {
                yield 1;
                resumed = true;
            }
            let iterator = stream();
            iterator.next().then(function () {
                iterator.throw(reason).then(undefined, function (error) {
                    print("throw:" + (error === reason) + ":" + resumed);
                    iterator.next().then(function (closed) {
                        print("closed:" + closed.value + ":" + closed.done);
                    });
                });
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("promise throw into suspended async-generator yield should run: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "throw:true:false".to_string(),
                "closed:undefined:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_promise_fulfillment_chains_through_default_and_callable_reactions() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            new Promise(function (resolve) { resolve(7); })
                .then()
                .then(function (value) { return value + 1; })
                .then(function (value) { print("value:" + value); });
            let rejected = new Promise(function (_, reject) { reject("bad"); });
            let propagated = rejected.then();
            propagated.then(undefined, function (reason) { print("reason:" + reason); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("fulfilled Promise chain should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync".to_string(),
                "reason:bad".to_string(),
                "value:8".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_promise_thrown_reaction_rejects_the_chained_promise() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            new Promise(function (_, reject) { reject("bad"); })
                .then(undefined, function (reason) { throw reason + "!"; })
                .then(undefined, function (reason) { print("caught:" + reason); });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("thrown Promise reaction should reject: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["caught:bad!".to_string()]
        );
    }

    #[test]
    fn wasm_backend_promise_catch_invokes_observable_then_once() {
        let source = r#"
            let catchMethod = Promise.prototype.catch;
            let receiver = {};
            let onRejected = function () {};
            let returnValue = {};
            let getCount = 0;
            let callCount = 0;
            let callWasExact = false;
            Object.defineProperty(receiver, "then", {
                get: function () {
                    getCount += 1;
                    return function (onFulfilled, rejectionHandler) {
                        callCount += 1;
                        callWasExact = this === receiver
                            && arguments.length === 2
                            && onFulfilled === undefined
                            && rejectionHandler === onRejected;
                        return returnValue;
                    };
                }
            });
            catchMethod.call(receiver, onRejected) === returnValue
                && getCount === 1
                && callCount === 1
                && callWasExact;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("Promise catch observable invocation should run: {err:?}")
            });
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_catch_propagates_lookup_and_call_errors() {
        let source = r#"
            let catchMethod = Promise.prototype.catch;
            let lookupError = {};
            let callError = {};
            let lookupResult;
            let callResult;
            let nullThrows = false;
            try { catchMethod.call(null); } catch (error) {
                nullThrows = error instanceof TypeError;
            }
            let poisoned = {};
            Object.defineProperty(poisoned, "then", {
                get: function () { throw lookupError; }
            });
            try { catchMethod.call(poisoned); } catch (error) {
                lookupResult = error;
            }
            try {
                catchMethod.call({
                    then: function () { throw callError; }
                });
            } catch (error) {
                callResult = error;
            }
            nullThrows && lookupResult === lookupError && callResult === callError;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise catch errors should propagate: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_catch_boxes_object_coercible_receivers() {
        let source = r#"
            let catchMethod = Promise.prototype.catch;
            let booleanCount = 0;
            let numberCount = 0;
            let stringCount = 0;
            let symbolCount = 0;
            Boolean.prototype.then = function () { booleanCount += 1; };
            Number.prototype.then = function () { numberCount += 1; };
            String.prototype.then = function () { stringCount += 1; };
            Symbol.prototype.then = function () { symbolCount += 1; };
            catchMethod.call(true);
            catchMethod.call(34);
            catchMethod.call("");
            catchMethod.call(Symbol());
            booleanCount === 1
                && numberCount === 1
                && stringCount === 1
                && symbolCount === 1;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise catch should box primitive receivers: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_catch_chains_rejections_asynchronously() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let promise = Promise.reject("bad");
            let chained = promise.catch(function (reason) {
                print("caught:" + reason);
                return 12;
            });
            chained.then(function (value) { print("value:" + value); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise catch rejection chain should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync".to_string(),
                "caught:bad".to_string(),
                "value:12".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_promise_finally_preserves_settlement_and_awaits_cleanup() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            Promise.resolve("value")
                .finally(function () {
                    print("cleanup:value");
                    return Promise.resolve("ignored");
                })
                .then(function (value) { print("fulfilled:" + value); });
            Promise.reject("reason")
                .finally(function () { print("cleanup:reason"); })
                .catch(function (reason) { print("rejected:" + reason); });
            Promise.resolve("value")
                .finally(function () { throw "replacement"; })
                .catch(function (reason) { print("replacement:" + reason); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise finally settlement should run: {err:?}"));
        let lines = lines.lock().expect("capture mutex poisoned");
        assert_eq!(lines.first(), Some(&"sync".to_string()));
        for expected in [
            "cleanup:value",
            "cleanup:reason",
            "replacement:replacement",
            "rejected:reason",
            "fulfilled:value",
        ] {
            assert!(
                lines.iter().any(|line| line == expected),
                "missing {expected:?}: {lines:?}"
            );
        }
    }

    #[test]
    fn wasm_backend_promise_finally_invokes_observable_then() {
        let source = r#"
            let finallyMethod = Promise.prototype.finally;
            let receiver = { constructor: Promise };
            let onFinally = function () {};
            let returnValue = {};
            let thenFinally;
            let catchFinally;
            receiver.then = function (first, second) {
                thenFinally = first;
                catchFinally = second;
                return returnValue;
            };
            let callableResult = finallyMethod.call(receiver, onFinally);
            let marker = {};
            let nonCallableResult = finallyMethod.call({
                constructor: Promise,
                then: function (first, second) {
                    return first === marker && second === marker;
                }
            }, marker);
            callableResult === returnValue
                && thenFinally !== onFinally
                && catchFinally !== onFinally
                && typeof thenFinally === "function"
                && typeof catchFinally === "function"
                && nonCallableResult === true;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("Promise finally observable invocation should run: {err:?}")
            });
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_species_getter_has_standard_descriptor_and_returns_receiver() {
        let source = r#"
            let descriptor = Object.getOwnPropertyDescriptor(Promise, Symbol.species);
            let receiver = {};
            Promise[Symbol.species] = {};
            descriptor.get.call(receiver) === receiver
                && Promise[Symbol.species] === Promise
                && descriptor.set === undefined
                && descriptor.enumerable === false
                && descriptor.configurable === true
                && descriptor.get.name === "get [Symbol.species]"
                && descriptor.get.length === 0;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise species descriptor should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_prototype_has_standard_to_string_tag_descriptor() {
        let source = r#"
            let descriptor = Object.getOwnPropertyDescriptor(
                Promise.prototype,
                Symbol.toStringTag
            );
            Promise.prototype[Symbol.toStringTag] = "changed";
            descriptor.value === "Promise"
                && descriptor.writable === false
                && descriptor.enumerable === false
                && descriptor.configurable === true
                && Promise.prototype[Symbol.toStringTag] === "Promise";
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise prototype toStringTag should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_then_uses_default_constructor_for_nullish_species() {
        let source = r#"
            let constructorReads = 0;
            let defaultConstructor = new Promise(function () {});
            Object.defineProperty(defaultConstructor, "constructor", {
                get: function () {
                    constructorReads += 1;
                    return Promise;
                }
            });
            let undefinedConstructorReads = 0;
            let undefinedConstructor = new Promise(function () {});
            Object.defineProperty(undefinedConstructor, "constructor", {
                get: function () {
                    undefinedConstructorReads += 1;
                    return undefined;
                }
            });

            let nullSpeciesReads = 0;
            let nullSpeciesConstructor = {};
            Object.defineProperty(nullSpeciesConstructor, Symbol.species, {
                get: function () {
                    nullSpeciesReads += 1;
                    return null;
                }
            });
            let nullSpecies = new Promise(function () {});
            nullSpecies.constructor = nullSpeciesConstructor;

            let undefinedSpeciesConstructor = {};
            undefinedSpeciesConstructor[Symbol.species] = undefined;
            let undefinedSpecies = new Promise(function () {});
            undefinedSpecies.constructor = undefinedSpeciesConstructor;

            let first = defaultConstructor.then();
            let second = undefinedConstructor.then();
            let third = nullSpecies.then();
            let fourth = undefinedSpecies.then();
            constructorReads === 1
                && undefinedConstructorReads === 1
                && nullSpeciesReads === 1
                && first instanceof Promise
                && second instanceof Promise
                && third instanceof Promise
                && fourth instanceof Promise;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("nullish Promise species defaults should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_then_propagates_constructor_and_species_errors() {
        let source = r#"
            let constructorError = {};
            let speciesError = {};
            let poisonedConstructor = new Promise(function () {});
            Object.defineProperty(poisonedConstructor, "constructor", {
                get: function () { throw constructorError; }
            });
            let poisonedSpeciesConstructor = {};
            Object.defineProperty(poisonedSpeciesConstructor, Symbol.species, {
                get: function () { throw speciesError; }
            });
            let poisonedSpecies = new Promise(function () {});
            poisonedSpecies.constructor = poisonedSpeciesConstructor;
            let nullConstructor = new Promise(function () {});
            nullConstructor.constructor = null;
            let invalidSpecies = new Promise(function () {});
            invalidSpecies.constructor = {};
            invalidSpecies.constructor[Symbol.species] = {};
            let constructionError = {};
            let throwingSpecies = new Promise(function () {});
            throwingSpecies.constructor = {};
            throwingSpecies.constructor[Symbol.species] = function () {
                throw constructionError;
            };

            let constructorResult;
            let speciesResult;
            let constructionResult;
            let nullThrows = false;
            let invalidSpeciesThrows = false;
            try { poisonedConstructor.then(); } catch (error) { constructorResult = error; }
            try { poisonedSpecies.then(); } catch (error) { speciesResult = error; }
            try { nullConstructor.then(); } catch (error) {
                nullThrows = error instanceof TypeError;
            }
            try { invalidSpecies.then(); } catch (error) {
                invalidSpeciesThrows = error instanceof TypeError;
            }
            try { throwingSpecies.then(); } catch (error) { constructionResult = error; }
            constructorResult === constructorError
                && speciesResult === speciesError
                && constructionResult === constructionError
                && nullThrows
                && invalidSpeciesThrows;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise species errors should propagate: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_then_uses_generic_custom_capabilities() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let returnedPromise = {};
            let constructorCalls = 0;
            function CapabilityConstructor(executor) {
                constructorCalls += 1;
                executor(
                    function (value) {
                        "use strict";
                        print("resolve:" + (this === undefined) + ":" + arguments.length + ":" + value);
                    },
                    function (reason) {
                        "use strict";
                        print("reject:" + (this === undefined) + ":" + arguments.length + ":" + (reason === expected));
                    }
                );
                return returnedPromise;
            }
            let constructor = {};
            constructor[Symbol.species] = CapabilityConstructor;
            let expected = {};

            let fulfilled = Promise.resolve(4);
            fulfilled.constructor = constructor;
            let fulfilledResult = fulfilled.then(function (value) { return value + 3; });

            let rejected = Promise.resolve(1);
            rejected.constructor = constructor;
            let rejectedResult = rejected.then(function () { throw expected; });

            print("returned:" + (fulfilledResult === returnedPromise));
            print("returned:" + (rejectedResult === returnedPromise));
            print("constructors:" + constructorCalls);
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generic Promise capabilities should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "returned:true".to_string(),
                "returned:true".to_string(),
                "constructors:2".to_string(),
                "sync".to_string(),
                "resolve:true:1:7".to_string(),
                "reject:true:1:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_promise_then_rejects_invalid_capability_initialization() {
        let source = r#"
            let missingConstructor = {};
            missingConstructor[Symbol.species] = function () { return {}; };
            let missing = new Promise(function () {});
            missing.constructor = missingConstructor;

            let repeatedConstructor = {};
            repeatedConstructor[Symbol.species] = function (executor) {
                executor(function () {}, function () {});
                executor(function () {}, function () {});
                return {};
            };
            let repeated = new Promise(function () {});
            repeated.constructor = repeatedConstructor;

            let missingThrows = false;
            let repeatedThrows = false;
            try { missing.then(); } catch (error) {
                missingThrows = error instanceof TypeError;
            }
            try { repeated.then(); } catch (error) {
                repeatedThrows = error instanceof TypeError;
            }
            missingThrows && repeatedThrows;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("invalid Promise capabilities should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_static_methods_validate_their_constructor_receiver() {
        let source = r#"
            let resolveThrows = false;
            let rejectThrows = false;
            try { Promise.resolve.call({}, 1); } catch (error) {
                resolveThrows = error instanceof TypeError;
            }
            try { Promise.reject.call(function () {}, 1); } catch (error) {
                rejectThrows = error instanceof TypeError;
            }
            resolveThrows && rejectThrows;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise static receiver validation should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_resolve_preserves_same_constructor_promise_identity() {
        let source = r#"
            let promise = new Promise(function () {});
            Promise.resolve(promise) === promise;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise.resolve identity should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_resolve_validates_receiver_before_identity_shortcut() {
        let source = r#"
            let promise = new Promise(function () {});
            let receivers = [undefined, null, true, 1, "", Symbol()];
            let allThrow = true;
            for (let receiver of receivers) {
                promise.constructor = receiver;
                try {
                    Promise.resolve.call(receiver, promise);
                    allThrow = false;
                } catch (error) {
                    allThrow = allThrow && error instanceof TypeError;
                }
            }
            allThrow;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("Promise.resolve receiver validation should run: {err:?}")
            });
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_with_resolvers_uses_generic_constructor_capability() {
        let source = r#"
            let returnedPromise = {};
            let expectedResolve = function () {};
            let expectedReject = function () {};
            let constructorCalls = 0;
            function Capability(executor) {
                constructorCalls += 1;
                executor(expectedResolve, expectedReject);
                return returnedPromise;
            }
            let capability = Promise.withResolvers.call(Capability);
            constructorCalls === 1
                && capability.promise === returnedPromise
                && capability.resolve === expectedResolve
                && capability.reject === expectedReject;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("Promise.withResolvers generic capability should run: {err:?}")
            });
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_with_resolvers_returns_standard_ordered_record() {
        let source = r#"
            let capability = Promise.withResolvers();
            let keys = Object.keys(capability);
            let promiseDescriptor = Object.getOwnPropertyDescriptor(capability, "promise");
            let resolveDescriptor = Object.getOwnPropertyDescriptor(capability, "resolve");
            let rejectDescriptor = Object.getOwnPropertyDescriptor(capability, "reject");
            let methodDescriptor = Object.getOwnPropertyDescriptor(Promise, "withResolvers");
            Object.getPrototypeOf(capability) === Object.prototype
                && keys.length === 3
                && keys[0] === "promise"
                && keys[1] === "resolve"
                && keys[2] === "reject"
                && promiseDescriptor.writable === true
                && promiseDescriptor.enumerable === true
                && promiseDescriptor.configurable === true
                && resolveDescriptor.writable === true
                && resolveDescriptor.enumerable === true
                && resolveDescriptor.configurable === true
                && rejectDescriptor.writable === true
                && rejectDescriptor.enumerable === true
                && rejectDescriptor.configurable === true
                && methodDescriptor.writable === true
                && methodDescriptor.enumerable === false
                && methodDescriptor.configurable === true
                && Promise.withResolvers.name === "withResolvers"
                && Promise.withResolvers.length === 0;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("Promise.withResolvers ordered record should run: {err:?}")
            });
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_with_resolvers_rejects_invalid_receivers() {
        let source = r#"
            let receivers = [undefined, null, 86, "string", true, Symbol(), () => {}];
            let allThrow = true;
            for (let receiver of receivers) {
                try {
                    Promise.withResolvers.call(receiver);
                    allThrow = false;
                } catch (error) {
                    allThrow = allThrow && error instanceof TypeError;
                }
            }
            allThrow;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("Promise.withResolvers invalid receivers should run: {err:?}")
            });
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_try_forwards_arguments_with_undefined_this() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            Promise.try(function (first, second, third) {
                "use strict";
                let received = Array.prototype.slice.call(arguments);
                print("callback:" + (this === undefined) + ":" + received.length + ":" + received[0] + ":" + received[1] + ":" + received[2]);
                return first + second + third;
            }, 2, 3, 4).then(function (value) { print("resolved:" + value); });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise.try argument forwarding should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "callback:true:3:2:3:4".to_string(),
                "sync".to_string(),
                "resolved:9".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_promise_try_rejects_callback_abrupt_completions() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let marker = {};
            Promise.try(function () { throw marker; }).catch(function (reason) {
                print("thrown:" + (reason === marker));
            });
            Promise.try(null).catch(function (reason) {
                print("invalid:" + (reason instanceof TypeError));
            });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("Promise.try abrupt completion rejection should run: {err:?}")
            });
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync".to_string(),
                "thrown:true".to_string(),
                "invalid:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_promise_try_uses_generic_constructor_capability() {
        let source = r#"
            let returnedPromise = {};
            let sentinel = {};
            let constructorCalls = 0;
            let callbackSawConstructor = false;
            let resolvedValue;
            let resolveThis;
            let rejectCalls = 0;
            function Capability(executor) {
                constructorCalls += 1;
                executor(function (value) {
                    "use strict";
                    resolveThis = this;
                    resolvedValue = value;
                }, function () { rejectCalls += 1; });
                return returnedPromise;
            }
            let result = Promise.try.call(Capability, function () {
                callbackSawConstructor = constructorCalls === 1;
                return sentinel;
            });
            result === returnedPromise
                && callbackSawConstructor
                && resolvedValue === sentinel
                && resolveThis === undefined
                && rejectCalls === 0;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise.try generic capability should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_try_validates_receiver_before_callback() {
        let source = r#"
            let callbackCalls = 0;
            let callback = function () { callbackCalls += 1; };
            let receivers = [undefined, null, true, 1, "", Symbol(), {}, () => {}];
            let allThrow = true;
            for (let receiver of receivers) {
                try {
                    Promise.try.call(receiver, callback);
                    allThrow = false;
                } catch (error) {
                    allThrow = allThrow && error instanceof TypeError;
                }
            }
            allThrow && callbackCalls === 0;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise.try receiver validation should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_static_settlement_schedules_reactions_after_synchronous_code() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            Promise.resolve(5).then(function (value) { print("resolved:" + value); });
            Promise.reject("bad").then(undefined, function (reason) {
                print("rejected:" + reason);
            });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise static settlement should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync".to_string(),
                "resolved:5".to_string(),
                "rejected:bad".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_promise_all_consumes_iterables_and_preserves_input_order() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let iterable = {};
            iterable[Symbol.iterator] = function () {
                let values = [Promise.resolve("first"), "second"];
                let index = 0;
                return {
                    next: function () {
                        if (index === values.length) return { done: true };
                        return { done: false, value: values[index++] };
                    }
                };
            };
            print("sync");
            Promise.all(iterable).then(function (values) {
                print(values.join(","));
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise.all iterable should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["sync".to_string(), "first,second".to_string()]
        );
    }

    #[test]
    fn wasm_backend_promise_all_settles_the_custom_constructor_capability() {
        let source = r#"
            let resolved;
            let capabilityObject;
            function CustomPromise(executor) {
                capabilityObject = {};
                executor(function (value) { resolved = value; }, function () {});
                return capabilityObject;
            }
            CustomPromise.resolve = function (value) {
                return { then: function (onFulfilled) { onFulfilled(value); } };
            };
            let result = Promise.all.call(CustomPromise, [1, 2]);
            result === capabilityObject &&
                Array.isArray(resolved) &&
                resolved[0] === 1 &&
                resolved[1] === 2;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("custom Promise.all capability should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_all_returns_a_derived_promise_instance() {
        let source = r#"
            class SubPromise extends Promise {}
            let instance = Promise.all.call(SubPromise, []);
            instance.constructor === SubPromise && instance instanceof SubPromise;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("derived Promise.all result should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_all_settled_preserves_input_order_and_record_descriptors() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            Promise.allSettled([
                Promise.reject("first"),
                Promise.resolve("second"),
                3
            ]).then(function (results) {
                let statusDescriptor = Object.getOwnPropertyDescriptor(results[0], "status");
                let reasonDescriptor = Object.getOwnPropertyDescriptor(results[0], "reason");
                let valueDescriptor = Object.getOwnPropertyDescriptor(results[1], "value");
                print(
                    results[0].status + ":" + results[0].reason + "," +
                    results[1].status + ":" + results[1].value + "," +
                    results[2].status + ":" + results[2].value + "," +
                    statusDescriptor.writable + ":" + statusDescriptor.enumerable + ":" + statusDescriptor.configurable + "," +
                    reasonDescriptor.writable + ":" + valueDescriptor.writable
                );
            });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise.allSettled records should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync".to_string(),
                "rejected:first,fulfilled:second,fulfilled:3,true:true:true,true:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_promise_all_settled_uses_one_resolve_lookup_and_a_shared_element_guard() {
        let source = r#"
            let capabilityObject = {};
            let resolved;
            let rejected;
            let resolveGets = 0;
            let resolveCalls = 0;
            function CustomPromise(executor) {
                executor(
                    function (value) { resolved = value; },
                    function (reason) { rejected = reason; }
                );
                return capabilityObject;
            }
            Object.defineProperty(CustomPromise, "resolve", {
                configurable: true,
                get: function () {
                    resolveGets += 1;
                    return function (value) {
                        resolveCalls += 1;
                        return {
                            then: function (onFulfilled, onRejected) {
                                onFulfilled(value);
                                onRejected("late rejection");
                                onFulfilled("late fulfillment");
                            }
                        };
                    };
                }
            });
            let result = Promise.allSettled.call(CustomPromise, [1, 2]);
            result === capabilityObject
                && rejected === undefined
                && resolveGets === 1
                && resolveCalls === 2
                && resolved.length === 2
                && resolved[0].status === "fulfilled"
                && resolved[0].value === 1
                && resolved[1].status === "fulfilled"
                && resolved[1].value === 2;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("custom Promise.allSettled should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_all_settled_closes_only_iterators_left_open_by_abrupt_completion() {
        let source = r#"
            let resolveError = {};
            let nextError = {};
            let closedAfterResolveError = 0;
            let closedAfterNextError = 0;
            let rejectedAfterResolveError;
            let rejectedAfterNextError;
            function capabilityFor(setReason) {
                return function (executor) {
                    executor(function () {}, setReason);
                    return {};
                };
            }
            let ResolveThrowingPromise = capabilityFor(function (reason) {
                rejectedAfterResolveError = reason;
            });
            ResolveThrowingPromise.resolve = function () { throw resolveError; };
            let resolveThrowingIterable = {};
            resolveThrowingIterable[Symbol.iterator] = function () {
                return {
                    next: function () { return { done: false, value: 1 }; },
                    return: function () {
                        closedAfterResolveError += 1;
                        return {};
                    }
                };
            };
            Promise.allSettled.call(ResolveThrowingPromise, resolveThrowingIterable);

            let NextThrowingPromise = capabilityFor(function (reason) {
                rejectedAfterNextError = reason;
            });
            NextThrowingPromise.resolve = function (value) { return Promise.resolve(value); };
            let nextThrowingIterable = {};
            nextThrowingIterable[Symbol.iterator] = function () {
                return {
                    next: function () { throw nextError; },
                    return: function () {
                        closedAfterNextError += 1;
                        return {};
                    }
                };
            };
            Promise.allSettled.call(NextThrowingPromise, nextThrowingIterable);

            rejectedAfterResolveError === resolveError
                && rejectedAfterNextError === nextError
                && closedAfterResolveError === 1
                && closedAfterNextError === 0;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("Promise.allSettled iterator closing should run: {err:?}")
            });
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_all_settled_returns_a_derived_empty_promise_without_species() {
        let source = r#"
            let speciesGets = 0;
            class SubPromise extends Promise {
                static get [Symbol.species]() {
                    speciesGets += 1;
                    throw new Error("species must not be read");
                }
            }
            let instance = Promise.allSettled.call(SubPromise, []);
            speciesGets === 0
                && instance.constructor === SubPromise
                && instance instanceof SubPromise;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("derived Promise.allSettled should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_all_keyed_preserves_own_keys_in_a_null_prototype_object() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let symbol = Symbol("symbol key");
            let prototype = { inherited: Promise.resolve("ignored") };
            let promises = {
                first: Promise.resolve("one"),
                second: 2
            };
            Object.setPrototypeOf(promises, prototype);
            Object.defineProperty(promises, "hidden", {
                value: Promise.resolve("ignored"),
                enumerable: false
            });
            Object.defineProperty(promises, symbol, {
                value: Promise.resolve("three"),
                enumerable: true,
                configurable: true,
                writable: true
            });
            Promise.allKeyed(promises).then(function (result) {
                let firstDescriptor = Object.getOwnPropertyDescriptor(result, "first");
                print(
                    (Object.getPrototypeOf(result) === null) + ":" +
                    Reflect.ownKeys(result).length + ":" +
                    Object.keys(result).join(",") + ":" +
                    result.first + ":" + result.second + ":" + result[symbol] + ":" +
                    firstDescriptor.writable + ":" + firstDescriptor.enumerable + ":" +
                    firstDescriptor.configurable
                );
            });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise.allKeyed should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync".to_string(),
                "true:3:first,second:one:2:three:true:true:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_promise_all_settled_keyed_uses_one_resolve_lookup_and_shared_guards() {
        let source = r#"
            let capabilityObject = {};
            let resolved;
            let rejected;
            let resolveGets = 0;
            let resolveCalls = 0;
            function CustomPromise(executor) {
                executor(
                    function (value) { resolved = value; },
                    function (reason) { rejected = reason; }
                );
                return capabilityObject;
            }
            Object.defineProperty(CustomPromise, "resolve", {
                get: function () {
                    resolveGets += 1;
                    return function (value) {
                        resolveCalls += 1;
                        return {
                            then: function (onFulfilled, onRejected) {
                                if (value === "bad") {
                                    onRejected("failure");
                                    onFulfilled("late fulfillment");
                                } else {
                                    onFulfilled(value);
                                    onRejected("late rejection");
                                }
                            }
                        };
                    };
                }
            });
            let result = Promise.allSettledKeyed.call(CustomPromise, {
                first: "good",
                second: "bad"
            });
            result === capabilityObject
                && rejected === undefined
                && resolveGets === 1
                && resolveCalls === 2
                && Object.getPrototypeOf(resolved) === null
                && Object.keys(resolved).join(",") === "first,second"
                && resolved.first.status === "fulfilled"
                && resolved.first.value === "good"
                && resolved.second.status === "rejected"
                && resolved.second.reason === "failure";
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise.allSettledKeyed should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_all_keyed_rechecks_enumerability_and_rejects_abrupt_gets() {
        let source = r#"
            let error = {};
            let rejected;
            let events = [];
            function CustomPromise(executor) {
                executor(function () {}, function (reason) { rejected = reason; });
                return {};
            }
            CustomPromise.resolve = function (value) {
                events.push("resolve:" + value);
                return { then: function (onFulfilled) { onFulfilled(value); } };
            };
            let target = { skipped: 1, throwing: 2 };
            let promises = new Proxy(target, {
                ownKeys: function () {
                    events.push("ownKeys");
                    return ["skipped", "throwing"];
                },
                getOwnPropertyDescriptor: function (target, key) {
                    events.push("descriptor:" + key);
                    if (key === "skipped") return undefined;
                    return Object.getOwnPropertyDescriptor(target, key);
                },
                get: function (_, key) {
                    events.push("get:" + key);
                    throw error;
                }
            });
            Promise.allKeyed.call(CustomPromise, promises);
            rejected === error
                && events.join(",") ===
                    "ownKeys,descriptor:skipped,descriptor:throwing,get:throwing";
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise.allKeyed abrupt get should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_any_fulfills_first_and_aggregates_ordered_rejections() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            Promise.any([
                Promise.reject("ignored"),
                Promise.resolve("winner"),
                Promise.resolve("later")
            ]).then(function (value) { print("fulfilled:" + value); });
            Promise.any([
                Promise.reject("first"),
                Promise.reject("second")
            ]).catch(function (error) {
                let descriptor = Object.getOwnPropertyDescriptor(error, "errors");
                print(
                    "rejected:" + (error instanceof AggregateError) + ":" +
                    error.errors.join(",") + ":" +
                    descriptor.writable + ":" + descriptor.enumerable + ":" + descriptor.configurable
                );
            });
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise.any settlement should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync".to_string(),
                "fulfilled:winner".to_string(),
                "rejected:true:first,second:true:false:true".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_promise_any_reject_element_is_fresh_and_idempotent() {
        let source = r#"
            let capabilityObject = {};
            let resolved;
            let rejected;
            let resolveGets = 0;
            let rejectFunctions = [];
            function CustomPromise(executor) {
                executor(
                    function (value) { resolved = value; },
                    function (reason) { rejected = reason; }
                );
                return capabilityObject;
            }
            Object.defineProperty(CustomPromise, "resolve", {
                get: function () {
                    resolveGets += 1;
                    return function (value) {
                        return {
                            then: function (_, onRejected) {
                                rejectFunctions.push(onRejected);
                                onRejected(value);
                                onRejected("late");
                            }
                        };
                    };
                }
            });
            let result = Promise.any.call(CustomPromise, ["first", "second"]);
            result === capabilityObject
                && resolved === undefined
                && rejected instanceof AggregateError
                && rejected.errors.join(",") === "first,second"
                && resolveGets === 1
                && rejectFunctions.length === 2
                && rejectFunctions[0] !== rejectFunctions[1];
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("custom Promise.any should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_any_rejects_empty_iterables_without_species() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let speciesGets = 0;
            class SubPromise extends Promise {
                static get [Symbol.species]() {
                    speciesGets += 1;
                    return Promise;
                }
            }
            let result = Promise.any.call(SubPromise, []);
            print(
                "result:" + (result instanceof SubPromise) + ":" +
                (result.constructor === SubPromise) + ":" + speciesGets
            );
            result.catch(function (error) {
                print("empty:" + (error instanceof AggregateError) + ":" + error.errors.length);
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("empty Promise.any should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["result:true:true:0".to_string(), "empty:true:0".to_string(),]
        );
    }

    #[test]
    fn wasm_backend_promise_race_keeps_an_empty_iterable_pending() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            Promise.race([]).then(
                function () { print("settled"); },
                function () { print("settled"); }
            );
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("empty Promise.race should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["sync".to_string()]
        );
    }

    #[test]
    fn wasm_backend_promise_race_preserves_first_settlement() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let resolveFirst;
            let rejectSecond;
            let first = new Promise(function (resolve) { resolveFirst = resolve; });
            let second = new Promise(function (_, reject) { rejectSecond = reject; });
            Promise.race([first, second]).then(
                function (value) { print("resolved:" + value); },
                function (reason) { print("rejected:" + reason); }
            );
            rejectSecond("second");
            resolveFirst("first");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise.race first settlement should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["rejected:second".to_string()]
        );
    }

    #[test]
    fn wasm_backend_promise_static_methods_propagate_capability_errors() {
        let source = r#"
            let expected = new Error("capability");
            function ResolveThrows(executor) {
                return new Promise(function () {
                    executor(function () { throw expected; }, function () {});
                });
            }
            function RejectThrows(executor) {
                return new Promise(function () {
                    executor(function () {}, function () { throw expected; });
                });
            }
            let resolveError;
            let rejectError;
            try { Promise.resolve.call(ResolveThrows, 1); } catch (error) {
                resolveError = error;
            }
            try { Promise.reject.call(RejectThrows, 1); } catch (error) {
                rejectError = error;
            }
            resolveError === expected && rejectError === expected;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise capability errors should propagate: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_promise_thenable_jobs_are_asynchronous_and_settle_once() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let thenable = {
                then: function (resolve, reject) {
                    print("then");
                    resolve(7);
                    reject("late");
                    throw new Error("later");
                }
            };
            let promise = Promise.resolve(thenable);
            promise.then(
                function (value) { print("value:" + value); },
                function (reason) { print("reason:" + reason); }
            );
            print("sync");
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("Promise thenable job should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &[
                "sync".to_string(),
                "then".to_string(),
                "value:7".to_string(),
            ]
        );
    }

    #[test]
    fn wasm_backend_promise_resolution_rejects_abrupt_then_getters() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let expected = new Error("getter");
            let thenable = {};
            Object.defineProperty(thenable, "then", {
                get: function () { throw expected; }
            });
            let promise = Promise.resolve(thenable);
            promise.then(undefined, function (reason) {
                print(reason === expected);
            });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("abrupt then getter should reject: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["true".to_string()]
        );
    }

    #[test]
    fn wasm_backend_promise_resolution_calls_proxy_then_functions() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let then = new Proxy(function (resolve) { resolve(11); }, {});
            let promise = Promise.resolve({ then: then });
            promise.then(function (value) { print("value:" + value); });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("proxy then function should run: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["value:11".to_string()]
        );
    }

    #[test]
    fn wasm_backend_promise_resolution_rejects_self_resolution() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let resolve;
            let promise = new Promise(function (onFulfilled) {
                resolve = onFulfilled;
            });
            promise.then(undefined, function (reason) {
                print(reason instanceof TypeError);
            });
            resolve(promise);
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("self-resolution should reject: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["true".to_string()]
        );
    }

    #[test]
    fn wasm_backend_promise_reactions_assimilate_returned_thenables() {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let source = r#"
            let promise = Promise.resolve(1);
            let chained = promise.then(function () {
                return { then: function (resolve) { resolve(9); } };
            });
            chained.then(function (value) { print("value:" + value); });
        "#;
        engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("returned thenable should be assimilated: {err:?}"));
        assert_eq!(
            lines.lock().expect("capture mutex poisoned").as_slice(),
            &["value:9".to_string()]
        );
    }

    #[test]
    fn wasm_backend_rejects_phase_twenty_nine_remaining_delete_edges() {
        for source in [
            "Error.stack",
            "Function.prototype.toString.call({}); Error.stack;",
        ] {
            let err = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect_err("phase 29 delete tail should stay unsupported");
            assert!(
                err.message()
                    .contains("unsupported in porffor wasm-aot first slice")
                    || err.message().contains("parse error"),
                "source: {source}, err: {}",
                err.message()
            );
        }
    }

    #[test]
    fn wasm_backend_resumes_zero_suspension_generators_lazily_and_once() {
        let source = r#"
            let ran = false;
            function* lazy() {
                ran = true;
                return 42;
            }
            const iterator = lazy();
            const lazyBeforeNext = !ran;
            const first = iterator.next();
            const second = iterator.next();
            lazyBeforeNext && ran
                && first.value === 42 && first.done === true
                && second.value === undefined && second.done === true
                && Object.getPrototypeOf(iterator) === lazy.prototype;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("zero-suspension generator should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_resumes_linear_generators_at_each_yield() {
        let source = r#"
            let effects = "";
            function* sequence() {
                yield 1;
                effects += "a";
                yield 2;
                effects += "b";
                return yield 3;
            }
            const firstIterator = sequence();
            const secondIterator = sequence();
            const first = firstIterator.next(99);
            const independent = secondIterator.next();
            const second = firstIterator.next(7);
            const third = firstIterator.next(8);
            const returned = firstIterator.next(9);
            const completed = firstIterator.next();
            function* activation(parameter) {
                let local = parameter;
                yield local;
                local += arguments[0];
                yield local;
                return local;
            }
            const activationIterator = activation(4);
            const activationFirst = activationIterator.next();
            const activationSecond = activationIterator.next();
            const activationReturn = activationIterator.next();
            let sent;
            function* injection() {
                sent = yield 5;
                yield sent;
            }
            const injectionIterator = injection();
            const injectionFirst = injectionIterator.next(100);
            const injectionSecond = injectionIterator.next(6);
            first.value === 1 && first.done === false
                && independent.value === 1 && independent.done === false
                && second.value === 2 && second.done === false
                && third.value === 3 && third.done === false
                && returned.value === 9 && returned.done === true
                && completed.value === undefined && completed.done === true
                && effects === "ab"
                && activationFirst.value === 4 && activationFirst.done === false
                && activationSecond.value === 8 && activationSecond.done === false
                && activationReturn.value === 8 && activationReturn.done === true
                && injectionFirst.value === 5 && injectionFirst.done === false
                && injectionSecond.value === 6 && injectionSecond.done === false
                && sent === 6;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("linear generator should resume: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_yields_regexp_literal_and_assigns_resumed_value() {
        let source = r#"
            let received;
            const sent = {};
            function* sequence() {
                received = yield/abc/i;
            }
            const iterator = sequence();
            const yielded = iterator.next();
            const completed = iterator.next(sent);
            yielded.value.test("ABC") === true && yielded.done === false
                && completed.value === undefined && completed.done === true
                && received === sent;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("regexp generator yield should resume: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_resumes_nested_yield_operands() {
        let source = r#"
            function* nested() {
                yield yield 1;
            }
            const iterator = nested();
            const inner = iterator.next();
            const outer = iterator.next(3);
            const completed = iterator.next(4);
            inner.value === 1 && inner.done === false
                && outer.value === 3 && outer.done === false
                && completed.value === undefined && completed.done === true;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("nested generator yield should resume: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_resumes_generator_return_call_arguments() {
        let source = r#"
            let calls = 0;
            const generator = function* g() {
                calls += 1;
                return (function(value) {
                    const yield = value + 1;
                    return yield;
                }(yield));
            };
            const iterator = generator();
            const suspended = iterator.next();
            const completed = iterator.next(42);
            suspended.value === undefined && suspended.done === false
                && completed.value === 43 && completed.done === true
                && calls === 1;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator return argument should resume: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_resumes_generator_object_spreads_in_source_order() {
        let source = r#"
            const symbol = Symbol("marker");
            let calls = 0;
            const generator = function* g() {
                calls += 1;
                yield {
                    ...yield yield,
                    ...(function(value) {
                        const yield = value;
                        return {...yield};
                    }(yield)),
                    ...yield,
                };
            };
            const iterator = generator();
            iterator.next();
            iterator.next();
            iterator.next({x: 10, a: 0, b: 0, [symbol]: 1});
            iterator.next({y: 20, a: 1, b: 1, [symbol]: 42});
            const yielded = iterator.next({z: 30, b: 2});
            const value = yielded.value;
            yielded.done === false && calls === 1
                && value.x === 10 && value.y === 20 && value.z === 30
                && value.a === 1 && value.b === 2 && value[symbol] === 42
                && Object.getOwnPropertySymbols(value)[0] === symbol
                && Object.keys(value).length === 5;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator object spreads should resume: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_resumes_generator_array_and_mixed_object_spreads() {
        let source = r#"
            function* singleArray() { yield [...yield]; }
            function* nestedArray() { yield [...yield yield]; }
            function* mixedObject() { yield {...yield, y: 1, ...yield yield}; }

            const single = singleArray();
            single.next();
            const singleValue = single.next(["a", "b", "c"]).value;

            const nested = nestedArray();
            nested.next();
            const nestedMiddle = nested.next(["a", "b", "c"]);
            const nestedValue = nested.next(nestedMiddle.value).value;

            const mixed = mixedObject();
            mixed.next();
            mixed.next({x: 42});
            mixed.next({x: "ignored"});
            const mixedValue = mixed.next({y: 39}).value;

            singleValue !== nestedValue
                && singleValue.join("") === "abc"
                && nestedValue.join("") === "abc"
                && mixedValue.x === 42 && mixedValue.y === 39
                && Object.keys(mixedValue).length === 2;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator literal spreads should resume: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_resumes_discarded_generator_expressions() {
        let source = r#"
            function* grouping() { (yield 1); }
            function* array() { [yield 2]; }
            function* block() { { yield 3; } }
            function* comma() { yield 4, yield 5; }
            function* conditional() { (yield 6) ? yield 7 : yield 8; }
            function* omitted() { [yield]; }

            const groupingIterator = grouping();
            const groupingYield = groupingIterator.next();
            const groupingDone = groupingIterator.next();
            const arrayIterator = array();
            const arrayYield = arrayIterator.next();
            const arrayDone = arrayIterator.next();
            const blockIterator = block();
            const blockYield = blockIterator.next();
            const blockDone = blockIterator.next();
            const commaIterator = comma();
            const commaFirst = commaIterator.next();
            const commaSecond = commaIterator.next(40);
            const commaDone = commaIterator.next(50);
            const truthyIterator = conditional();
            const truthyCondition = truthyIterator.next();
            const truthyBranch = truthyIterator.next(true);
            const truthyDone = truthyIterator.next();
            const falsyIterator = conditional();
            const falsyCondition = falsyIterator.next();
            const falsyBranch = falsyIterator.next(false);
            const falsyDone = falsyIterator.next();
            const omittedIterator = omitted();
            const omittedYield = omittedIterator.next();
            const omittedDone = omittedIterator.next();

            groupingYield.value === 1 && groupingYield.done === false
                && groupingDone.value === undefined && groupingDone.done === true
                && arrayYield.value === 2 && arrayYield.done === false
                && arrayDone.value === undefined && arrayDone.done === true
                && blockYield.value === 3 && blockYield.done === false
                && blockDone.value === undefined && blockDone.done === true
                && commaFirst.value === 4 && commaFirst.done === false
                && commaSecond.value === 5 && commaSecond.done === false
                && commaDone.value === undefined && commaDone.done === true
                && truthyCondition.value === 6 && truthyCondition.done === false
                && truthyBranch.value === 7 && truthyBranch.done === false
                && truthyDone.value === undefined && truthyDone.done === true
                && falsyCondition.value === 6 && falsyCondition.done === false
                && falsyBranch.value === 8 && falsyBranch.done === false
                && falsyDone.value === undefined && falsyDone.done === true
                && omittedYield.value === undefined && omittedYield.done === false
                && omittedDone.value === undefined && omittedDone.done === true;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("discarded generator expressions should resume: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_resumes_template_interpolations_in_evaluation_order() {
        let source = r#"
            let order = "";
            let output;
            function coercible(text, marker) {
                return {
                    toString: function () {
                        order += marker;
                        return text;
                    }
                };
            }
            function* interpolate() {
                output = `a${coercible("b", "B")}${yield 1}d${coercible("e", "E")}${yield 2}g`;
            }

            const iterator = interpolate();
            const first = iterator.next();
            const afterFirst = output === undefined && order === "B";
            const second = iterator.next(coercible("c", "C"));
            const afterSecond = output === undefined && order === "BCE";
            const third = iterator.next(coercible("f", "F"));

            first.value === 1 && first.done === false
                && second.value === 2 && second.done === false
                && third.value === undefined && third.done === true
                && afterFirst && afterSecond
                && output === "abcdefg" && order === "BCEF";
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("generator template interpolation should resume: {err:?}")
            });
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_preserves_with_object_bindings_across_generator_resume() {
        let source = r#"
            let evaluations = 0;
            function createScope() {
                evaluations += 1;
                return { x: 2 };
            }
            function* sequence() {
                let x = 1;
                yield x;
                with (createScope()) {
                    yield x;
                    x = 3;
                    yield x;
                }
                yield x;
            }

            const iterator = sequence();
            const first = iterator.next();
            const second = iterator.next();
            const third = iterator.next();
            const fourth = iterator.next();
            const completed = iterator.next();

            first.value === 1 && first.done === false
                && second.value === 2 && second.done === false
                && third.value === 3 && third.done === false
                && fourth.value === 1 && fourth.done === false
                && completed.value === undefined && completed.done === true
                && evaluations === 1;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator with binding should resume: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_preserves_property_assignment_references_across_generator_yield() {
        let source = r#"
            let order = "";
            const original = {};
            const replacement = {};
            let selectedTarget = original;
            let selectedKey = "before";
            function assignmentTarget() {
                order += "target";
                return selectedTarget;
            }
            function assignmentKey() {
                order += ",key";
                return selectedKey;
            }
            function yieldedValue() {
                order += ",yield";
                return 1;
            }
            function* assigned() {
                assignmentTarget()[assignmentKey()] = yield yieldedValue();
            }
            const assignedIterator = assigned();
            const assignedYield = assignedIterator.next();
            selectedTarget = replacement;
            selectedKey = "after";
            const assignedResult = assignedIterator.next(7);

            function* interrupted() {
                try {
                    original.interrupted = yield 2;
                } finally {
                    return 9;
                }
            }
            const interruptedIterator = interrupted();
            const interruptedYield = interruptedIterator.next();
            const interruptedResult = interruptedIterator.return(45);

            order === "target,key,yield"
                && assignedYield.value === 1 && assignedYield.done === false
                && assignedResult.value === undefined && assignedResult.done === true
                && original.before === 7
                && !("after" in replacement)
                && interruptedYield.value === 2 && interruptedYield.done === false
                && interruptedResult.value === 9 && interruptedResult.done === true
                && !("interrupted" in original);
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator property assignment should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_initializes_generator_parameters_at_call_time() {
        let source = r#"
            let effects = "";
            function* initialized(value = (effects += "p", 3), ...rest) {
                effects += "b";
                yield value;
                yield rest[1];
            }
            const iterator = initialized(undefined, 4, 5);
            const initializedBeforeResume = effects === "p";
            const first = iterator.next();
            const second = iterator.next();

            function* destructured([left], { right }) {
                effects += "d";
                yield left + right;
            }
            const destructuredIterator = destructured([2], { right: 3 });
            const destructuredBeforeResume = effects === "pb";
            const destructuredYield = destructuredIterator.next();

            const intrinsicGeneratorPrototype = Object.getPrototypeOf(
                Object.getPrototypeOf((function* () {})())
            );
            function* topology(value = (topology.prototype = null)) { yield value; }
            const topologyIterator = topology();

            let thrownValue = 0;
            function* abrupt(value = (() => { throw 9; })()) {
                effects += "x";
                yield value;
            }
            try { abrupt(); } catch (error) { thrownValue = error; }

            initializedBeforeResume
                && effects === "pbd"
                && first.value === 3 && first.done === false
                && second.value === 5 && second.done === false
                && destructuredBeforeResume
                && destructuredYield.value === 5 && destructuredYield.done === false
                && Object.getPrototypeOf(topologyIterator) === intrinsicGeneratorPrototype
                && thrownValue === 9;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator parameters should initialize: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_exposes_generator_function_prototype_topology() {
        let source = r#"
            const generator = function* sample() {};
            const functionPrototype = Object.getPrototypeOf(generator);
            const generatorPrototype = Object.getPrototypeOf(generator.prototype);
            const constructor = functionPrototype.constructor;
            const ownPrototypeDescriptor = Object.getOwnPropertyDescriptor(generator, "prototype");
            const intrinsicPrototypeDescriptor = Object.getOwnPropertyDescriptor(
                functionPrototype,
                "prototype"
            );
            const generatorConstructorDescriptor = Object.getOwnPropertyDescriptor(
                generatorPrototype,
                "constructor"
            );
            let prototypeCallThrows = false;
            try { functionPrototype(); } catch (error) { prototypeCallThrows = error instanceof TypeError; }

            Object.getPrototypeOf(functionPrototype) === Function.prototype
                && Object.getPrototypeOf(constructor) === functionPrototype
                && constructor.prototype === functionPrototype
                && functionPrototype.prototype === generatorPrototype
                && generatorPrototype.constructor === functionPrototype
                && Object.getOwnPropertyNames(generator.prototype).length === 0
                && ownPrototypeDescriptor.writable === true
                && ownPrototypeDescriptor.enumerable === false
                && ownPrototypeDescriptor.configurable === false
                && intrinsicPrototypeDescriptor.writable === false
                && intrinsicPrototypeDescriptor.enumerable === false
                && intrinsicPrototypeDescriptor.configurable === true
                && generatorConstructorDescriptor.writable === false
                && generatorConstructorDescriptor.enumerable === false
                && generatorConstructorDescriptor.configurable === true
                && constructor.name === "GeneratorFunction"
                && constructor.length === 1
                && typeof functionPrototype === "object"
                && prototypeCallThrows;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator topology should compile: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_exposes_generator_expression_names() {
        let source = r#"
            const inferred = function* () {};
            const anonymous = [function* () {}][0];
            const explicit = function* named() {};
            const inferredDescriptor = Object.getOwnPropertyDescriptor(inferred, "name");
            const anonymousDescriptor = Object.getOwnPropertyDescriptor(anonymous, "name");
            const explicitDescriptor = Object.getOwnPropertyDescriptor(explicit, "name");

            inferredDescriptor.value === "inferred"
                && anonymousDescriptor.value === ""
                && explicitDescriptor.value === "named"
                && inferredDescriptor.writable === false
                && anonymousDescriptor.enumerable === false
                && explicitDescriptor.configurable === true;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator names should compile: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_supports_generator_function_has_instance() {
        let source = r#"
            const generator = function* () {};
            const arrow = () => {};
            let arrowThrows = false;
            try {
                ({} instanceof arrow);
            } catch (error) {
                arrowThrows = error instanceof TypeError;
            }

            generator() instanceof generator && arrowThrows;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator instanceof should compile: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_initializes_generator_default_parameters_in_tdz_order() {
        let source = r#"
            const selfRead = function* (value = value) {};
            const laterRead = function* (value = later, later) {};
            const priorRead = function* (value = 7, later = value) { yield later; };
            const destructuredLaterRead = function* ([value = later, later]) {};
            const destructuredPriorRead = function* ([value = 11, later = value]) { yield later; };

            let selfReadThrows = false;
            let laterReadThrows = false;
            let destructuredLaterReadThrows = false;
            try { selfRead(); } catch (error) { selfReadThrows = error instanceof ReferenceError; }
            try { laterRead(); } catch (error) { laterReadThrows = error instanceof ReferenceError; }
            try { destructuredLaterRead([]); } catch (error) {
                destructuredLaterReadThrows = error instanceof ReferenceError;
            }
            const priorResult = priorRead().next();
            const destructuredPriorResult = destructuredPriorRead([]).next();

            selfReadThrows
                && laterReadThrows
                && destructuredLaterReadThrows
                && priorResult.value === 7
                && priorResult.done === false
                && destructuredPriorResult.value === 11
                && destructuredPriorResult.done === false;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator parameter TDZ should compile: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_with_typed_array_reads_outer_infinity_binding() {
        let source = r#"
            function observe(view) {
                with (view) {
                    return Infinity === Infinity;
                }
            }

            const numeric = new Uint8Array([1]);
            const bigint = new BigInt64Array([1n]);
            __porfDetachArrayBuffer(numeric.buffer);
            __porfDetachArrayBuffer(bigint.buffer);
            observe(numeric) && observe(bigint);
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("typed-array with Infinity fallback should compile: {err:?}")
            });
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_with_typed_array_reads_outer_object_binding() {
        let source = r#"
            const fallback = { value: 17 };
            function observe(view) {
                with (view) {
                    return fallback.value === 17;
                }
            }

            const numeric = new Uint8Array([1]);
            __porfDetachArrayBuffer(numeric.buffer);
            observe(numeric);
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("typed-array with object fallback should compile: {err:?}")
            });
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_with_typed_array_reads_outer_function_binding() {
        let source = r#"
            function readFallback() { return 17; }
            function observe(view) {
                with (view) {
                    return readFallback() === 17;
                }
            }

            const numeric = new Uint8Array([1]);
            __porfDetachArrayBuffer(numeric.buffer);
            observe(numeric);
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("typed-array with function fallback should compile: {err:?}")
            });
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_honors_unscopables_for_generator_with_bindings() {
        let source = r#"
            globalThis.excluded = 9;
            globalThis.included = 10;
            globalThis.probePassed = false;
            globalThis[Symbol.unscopables] = { excluded: true };

            const probe = function* (input) {
                let observedHoistedVar;
                let observedExcluded;
                let observedIncluded;
                with (globalThis) {
                    observedHoistedVar = excluded;
                }
                var excluded = input;
                with (globalThis) {
                    observedExcluded = excluded;
                    observedIncluded = included;
                    excluded = 2;
                    included = 11;
                }
                globalThis.probePassed = observedHoistedVar === undefined
                    && observedExcluded === 1
                    && observedIncluded === 10
                    && excluded === 2
                    && globalThis.excluded === 9
                    && globalThis.included === 11;
            };

            probe(1).next();

            let count = 0;
            let observations = {};
            var v = 1;
            globalThis[Symbol.unscopables].v = true;
            count++;
            var callCount = 0;
            const activationProbe = function* (input) {
                count++;
                with (globalThis) {
                    count++;
                    observations.initial = v;
                }
                count++;
                var v = input;
                with (globalThis) {
                    count++;
                    observations.initialized = v;
                    v = 20;
                }
                observations.assigned = v;
                observations.global = globalThis.v;
                callCount++;
            };
            activationProbe(10).next();
            count++;

            globalThis.probePassed
                && observations.initial === undefined
                && observations.initialized === 10
                && observations.assigned === 20
                && observations.global === 1
                && callCount === 1
                && count === 6;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator unscopables should compile: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_enforces_named_generator_expression_binding_immutability() {
        for source in [
            r#"
                var readParameter, setParameter, readBody, setBody;
                var generator = function* binding(
                    unused = (
                        readParameter = function() { return binding; },
                        setParameter = function() { binding = null; }
                    )
                ) {
                    readBody = function() { return binding; };
                    setBody = function() { binding = null; };
                };
                generator().next();
                setParameter();
                setBody();
                readParameter() === generator && readBody() === generator;
            "#,
            r#"
                "use strict";
                var readParameter, setParameter, readBody, setBody;
                var generator = function* binding(
                    unused = (
                        readParameter = function() { return binding; },
                        setParameter = function() { binding = null; }
                    )
                ) {
                    readBody = function() { return binding; };
                    setBody = function() { binding = null; };
                };
                generator().next();
                var parameterThrew = false;
                var bodyThrew = false;
                try { setParameter(); } catch (error) {
                    parameterThrew = error instanceof TypeError;
                }
                try { setBody(); } catch (error) {
                    bodyThrew = error instanceof TypeError;
                }
                parameterThrew
                    && bodyThrew
                    && readParameter() === generator
                    && readBody() === generator;
            "#,
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|err| panic!("named generator binding should compile: {err:?}"));
            assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
        }
    }

    #[test]
    fn wasm_backend_separates_generator_parameter_and_body_var_environments() {
        let outcome = engine()
            .run_script(
                r#"
                    var x = "outside";
                    var readParameter;
                    var readBody;
                    (function* (
                        unused = readParameter = function() { return x; }
                    ) {
                        var x = "inside";
                        readBody = function() { return x; };
                    }().next());
                    readParameter() === "outside" && readBody() === "inside";
                "#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("generator parameter environment should compile: {err:?}")
            });
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_copies_generator_object_rest_parameters() {
        let outcome = engine()
            .run_script(
                r#"
                    var explicitRest;
                    var defaultRest;
                    var explicit = function* ({ a, b, ...rest }) {
                        explicitRest = rest;
                    };
                    var defaulted = function* (
                        { a, ...rest } = { a: 5, x: 6, y: 7 }
                    ) {
                        defaultRest = rest;
                    };
                    explicit({ a: 1, b: 2, x: 3, y: 4 }).next();
                    defaulted().next();
                    explicitRest.a === undefined
                        && explicitRest.b === undefined
                        && explicitRest.x === 3
                        && explicitRest.y === 4
                        && defaultRest.a === undefined
                        && defaultRest.x === 6
                        && defaultRest.y === 7
                        && Object.getPrototypeOf(explicitRest) === Object.prototype
                        && Object.getPrototypeOf(defaultRest) === Object.prototype;
                "#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator object rest should compile: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_evaluates_computed_generator_object_parameter_keys() {
        let outcome = engine()
            .run_script(
                r#"
                    var keyCalls = 0;
                    var defaultCalls = 0;
                    var firstValue;
                    var secondValue;
                    var first = function* ({ [keyCalls = keyCalls + 1]: value = 9 }) {
                        firstValue = value;
                    };
                    var second = function* ({ [keyCalls = keyCalls + 1]: value = (defaultCalls = defaultCalls + 1, 9) }) {
                        secondValue = value;
                    };
                    first({ 1: 7 }).next();
                    second({ 2: undefined }).next();

                    var marker = {};
                    var bodyRan = false;
                    var thrown;
                    function thrower() { throw marker; }
                    var abrupt = function* ({ [thrower()]: value }) {
                        bodyRan = true;
                    };
                    try { abrupt({}); } catch (error) { thrown = error; }

                    keyCalls === 2
                        && defaultCalls === 1
                        && firstValue === 7
                        && secondValue === 9
                        && thrown === marker
                        && bodyRan === false;
                "#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("computed generator object parameter keys should compile: {err:?}")
            });
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_destructures_nested_generator_parameters() {
        let outcome = engine()
            .run_script(
                r#"
                    var explicit;
                    var defaulted;
                    var f = function* (
                        [{ x, renamed: y }],
                        { values: [z] }
                    ) {
                        explicit = [x, y, z];
                    };
                    var g = function* (
                        [{ x } = { x: 4 }] = [],
                        { values: [y] } = { values: [5] }
                    ) {
                        defaulted = [x, y];
                    };
                    f([{ x: 1, renamed: 2 }], { values: [3] }).next();
                    g().next();
                    (explicit[0] === 1 ? 1 : 0)
                        + (explicit[1] === 2 ? 2 : 0)
                        + (explicit[2] === 3 ? 4 : 0)
                        + (defaulted[0] === 4 ? 8 : 0)
                        + (defaulted[1] === 5 ? 16 : 0);
                "#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("nested generator parameter patterns should compile: {err:?}")
            });
        assert!(outcome.note.contains("number(31)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_injects_return_and_throw_into_suspended_generators() {
        let source = r#"
            let returnEffects = "";
            function* returnSequence() {
                returnEffects += "a";
                yield 1;
                returnEffects += "b";
                yield 2;
            }
            const returned = returnSequence();
            const returnFirst = returned.next();
            const returnResult = returned.return(9);
            const returnCompleted = returned.next();

            const marker = { marker: true };
            let thrownValue;
            function* throwSequence() {
                yield 3;
                yield 4;
            }
            const thrown = throwSequence();
            const throwFirst = thrown.next();
            try { thrown.throw(marker); } catch (error) { thrownValue = error; }
            const throwCompleted = thrown.next();

            (returnFirst.value === 1 && returnFirst.done === false ? 1 : 0)
                + (returnResult.value === 9 && returnResult.done === true ? 2 : 0)
                + (returnCompleted.value === undefined && returnCompleted.done === true ? 4 : 0)
                + (returnEffects === "a" ? 8 : 0)
                + (throwFirst.value === 3 && throwFirst.done === false ? 16 : 0)
                + (thrownValue === marker ? 32 : 0)
                + (thrownValue && thrownValue.marker === true ? 64 : 0)
                + (throwCompleted.value === undefined ? 128 : 0)
                + (throwCompleted.done === true ? 256 : 0);
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("abrupt generator resume should compile: {err:?}"));
        assert!(outcome.note.contains("number(511)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_preserves_abrupt_completions_across_yielding_generator_handlers() {
        let source = r#"
            const caughtMarker = { caught: true };
            function* caught() {
                try {
                    yield 1;
                } catch (error) {
                    yield error;
                    return 2;
                }
            }
            const caughtIterator = caught();
            const caughtFirst = caughtIterator.next();
            const caughtThrow = caughtIterator.throw(caughtMarker);
            const caughtReturn = caughtIterator.next();

            let returnEffects = "";
            function* returned() {
                try {
                    yield 3;
                } finally {
                    returnEffects += "f";
                    yield 4;
                    returnEffects += "g";
                }
            }
            const returnedIterator = returned();
            const returnedFirst = returnedIterator.next();
            const returnedFinally = returnedIterator.return(5);
            const returnedResult = returnedIterator.next();

            const thrownMarker = { thrown: true };
            function* thrown() {
                try {
                    yield 6;
                } finally {
                    yield 7;
                }
            }
            const thrownIterator = thrown();
            const thrownFirst = thrownIterator.next();
            const thrownFinally = thrownIterator.throw(thrownMarker);
            let thrownResult;
            try {
                thrownIterator.next();
            } catch (error) {
                thrownResult = error;
            }

            const overrideMarker = { override: true };
            function* overridden() {
                try {
                    yield 8;
                } finally {
                    throw overrideMarker;
                }
            }
            const overriddenIterator = overridden();
            const overriddenFirst = overriddenIterator.next();
            let overrideResult;
            try {
                overriddenIterator.return(9);
            } catch (error) {
                overrideResult = error;
            }
            const overriddenCompleted = overriddenIterator.next();

            function* nested() {
                try {
                    try {
                        yield 10;
                    } finally {
                        yield 11;
                    }
                } finally {
                    yield 12;
                }
            }
            const nestedIterator = nested();
            const nestedFirst = nestedIterator.next();
            const nestedInnerFinally = nestedIterator.return(13);
            const nestedOuterFinally = nestedIterator.next();
            const nestedResult = nestedIterator.next();

            caughtFirst.value === 1 && caughtFirst.done === false
                && caughtThrow.value === caughtMarker && caughtThrow.done === false
                && caughtReturn.value === 2 && caughtReturn.done === true
                && returnedFirst.value === 3 && returnedFirst.done === false
                && returnedFinally.value === 4 && returnedFinally.done === false
                && returnedResult.value === 5 && returnedResult.done === true
                && returnEffects === "fg"
                && thrownFirst.value === 6 && thrownFirst.done === false
                && thrownFinally.value === 7 && thrownFinally.done === false
                && thrownResult === thrownMarker
                && overriddenFirst.value === 8 && overriddenFirst.done === false
                && overrideResult === overrideMarker
                && overriddenCompleted.value === undefined && overriddenCompleted.done === true
                && nestedFirst.value === 10 && nestedFirst.done === false
                && nestedInnerFinally.value === 11 && nestedInnerFinally.done === false
                && nestedOuterFinally.value === 12 && nestedOuterFinally.done === false
                && nestedResult.value === 13 && nestedResult.done === true;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("structured generator handlers should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_resumes_generators_inside_simple_loops_without_replaying_work() {
        let source = r#"
            let forEffects = "";
            function* forSequence() {
                for (var i = 0; i < 3; i++) {
                    forEffects += "b";
                    yield i;
                    forEffects += "a";
                }
                return i;
            }
            const forIterator = forSequence();
            const forFirst = forIterator.next();
            const forSecond = forIterator.next();
            const forThird = forIterator.next();
            const forReturn = forIterator.next();

            function* whileSequence() {
                let i = 0;
                while (i < 2) {
                    yield i;
                    i++;
                }
                return i;
            }
            const whileIterator = whileSequence();
            const whileFirst = whileIterator.next();
            const whileSecond = whileIterator.next();
            const whileReturn = whileIterator.next();

            forFirst.value === 0 && forFirst.done === false
                && forSecond.value === 1 && forSecond.done === false
                && forThird.value === 2 && forThird.done === false
                && forReturn.value === 3 && forReturn.done === true
                && forEffects === "bababa"
                && whileFirst.value === 0 && whileFirst.done === false
                && whileSecond.value === 1 && whileSecond.done === false
                && whileReturn.value === 2 && whileReturn.done === true;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("loop generator should resume: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_resumes_the_selected_generator_branch_without_replaying_its_prefix() {
        let source = r#"
            let effects = "";
            function* choose(flag) {
                if (flag) {
                    effects += "T";
                    yield 1;
                    effects += "t";
                } else {
                    effects += "F";
                    yield 2;
                    effects += "f";
                }
                return 3;
            }
            const truthy = choose(true);
            const falsy = choose(false);
            const truthyYield = truthy.next();
            const falsyYield = falsy.next();
            const truthyReturn = truthy.next();
            const falsyReturn = falsy.next();

            function* maybe(flag) {
                if (flag) yield 4;
                return 5;
            }
            const skipped = maybe(false).next();

            function* conditional(flag) {
                yield flag ? 6 : 7;
                return 8;
            }
            const conditionalIterator = conditional(false);
            const conditionalYield = conditionalIterator.next();
            const conditionalReturn = conditionalIterator.next();

            truthyYield.value === 1 && truthyYield.done === false
                && falsyYield.value === 2 && falsyYield.done === false
                && truthyReturn.value === 3 && truthyReturn.done === true
                && falsyReturn.value === 3 && falsyReturn.done === true
                && skipped.value === 5 && skipped.done === true
                && conditionalYield.value === 7 && conditionalYield.done === false
                && conditionalReturn.value === 8 && conditionalReturn.done === true
                && effects === "TFtf";
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("branch generator should resume: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_preserves_generator_captures_across_yield() {
        let source = r#"
            let lexicalLet = { source: "let" };
            const lexicalConst = { source: "const" };
            var scriptVar = { source: "var" };
            const parameterValue = { source: "parameter" };

            function* captureLet() { yield 0; return lexicalLet; }
            function* captureConst() { yield 0; return lexicalConst; }
            function* captureVar() { yield 0; return scriptVar; }
            function* captureParameter(value) { yield 0; return value; }

            var letIterator = captureLet();
            var constIterator = captureConst();
            var varIterator = captureVar();
            var parameterIterator = captureParameter(parameterValue);
            letIterator.next();
            constIterator.next();
            varIterator.next();
            parameterIterator.next();

            letIterator.next().value === lexicalLet
                && constIterator.next().value === lexicalConst
                && varIterator.next().value === scriptVar
                && parameterIterator.next().value === parameterValue;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator captures should resume: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_forwards_generator_delegation_protocol_completions() {
        let source = r#"
            var nextGets = 0;
            var valueGets = 0;
            var normalFinal;
            var nextArgs = [];
            var firstResult = Object.defineProperty({}, "value", {
                get() { valueGets++; return 1; }
            });
            var normalStep = 0;
            var normalIterable = {
                [Symbol.iterator]() {
                    return {
                        get next() {
                            nextGets++;
                            return function (value) {
                                nextArgs.push(value);
                                normalStep++;
                                if (normalStep === 1) return firstResult;
                                return { done: true, value: 9 };
                            };
                        }
                    };
                }
            };
            function* normal() { normalFinal = yield* normalIterable; }
            var normalIterator = normal();
            var normalFirst = normalIterator.next(99);
            var normalDone = normalIterator.next(7);

            var returnedRaw = { done: false, value: 5 };
            var returnReceived;
            var returnNextReceived;
            var returnIterable = {
                [Symbol.iterator]() {
                    return {
                        next(value) {
                            returnNextReceived = value;
                            return value === 6
                                ? { done: true, value: 8 }
                                : { done: false, value: 1 };
                        },
                        return(value) {
                            returnReceived = value;
                            return returnedRaw;
                        }
                    };
                }
            };
            function* returned() { return yield* returnIterable; }
            var returnedIterator = returned();
            returnedIterator.next();
            var returnedYield = returnedIterator.return(4);
            var returnedDone = returnedIterator.next(6);

            var thrownRaw = { done: false, value: 12 };
            var throwReceived;
            var throwFinal;
            var throwStep = 0;
            var throwIterable = {
                [Symbol.iterator]() {
                    return {
                        next() {
                            throwStep++;
                            return throwStep === 1
                                ? { done: false, value: 10 }
                                : { done: true, value: 13 };
                        },
                        throw(value) {
                            throwReceived = value;
                            return thrownRaw;
                        }
                    };
                }
            };
            function* thrown() { throwFinal = yield* throwIterable; }
            var thrownIterator = thrown();
            thrownIterator.next();
            var thrownYield = thrownIterator.throw(11);
            var thrownDone = thrownIterator.next();

            var closeArgs = -1;
            var missingThrowCaught = false;
            var missingThrowIterable = {
                [Symbol.iterator]() {
                    return {
                        next() { return { done: false }; },
                        return() {
                            closeArgs = arguments.length;
                            return {};
                        }
                    };
                }
            };
            function* missingThrow() {
                try { yield* missingThrowIterable; }
                catch (error) { missingThrowCaught = error instanceof TypeError; }
            }
            var missingThrowIterator = missingThrow();
            missingThrowIterator.next();
            var missingThrowDone = missingThrowIterator.throw(14);

            var nonObjectCaught = false;
            var nonObjectIterable = {
                [Symbol.iterator]() { return { next() { return 1; } }; }
            };
            function* nonObject() {
                try { yield* nonObjectIterable; }
                catch (error) { nonObjectCaught = error instanceof TypeError; }
            }
            var nonObjectDone = nonObject().next();

            normalFirst === firstResult
                && normalFirst.done === undefined
                && valueGets === 0
                && nextGets === 1
                && nextArgs.length === 2
                && nextArgs[0] === undefined
                && nextArgs[1] === 7
                && normalFinal === 9
                && normalDone.value === undefined && normalDone.done === true
                && returnReceived === 4
                && returnedYield === returnedRaw
                && returnNextReceived === 6
                && returnedDone.value === 8 && returnedDone.done === true
                && throwReceived === 11
                && thrownYield === thrownRaw
                && throwFinal === 13
                && thrownDone.value === undefined && thrownDone.done === true
                && closeArgs === 0
                && missingThrowCaught
                && missingThrowDone.done === true
                && nonObjectCaught
                && nonObjectDone.done === true;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator delegation should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_generator_return_throw_and_brand_checks_match_iterator_semantics() {
        let source = r#"
            let ranReturn = false;
            function* returned() { ranReturn = true; }
            const returnedIterator = returned();
            const returnedResult = returnedIterator.return(7);

            let ranThrow = false;
            function* thrown() { ranThrow = true; }
            const thrownIterator = thrown();
            let thrownValue = 0;
            try { thrownIterator.throw(9); } catch (error) { thrownValue = error; }

            let incompatibleReceiverThrows = false;
            try { returnedIterator.next.call({}); } catch (error) {
                incompatibleReceiverThrows = error instanceof TypeError;
            }

            let reentrantIterator;
            function* reentrant() { reentrantIterator.next(); }
            reentrantIterator = reentrant();
            let reentrantThrows = false;
            try { reentrantIterator.next(); } catch (error) {
                reentrantThrows = error instanceof TypeError;
            }
            const afterReentrancy = reentrantIterator.next();

            !ranReturn && returnedResult.value === 7 && returnedResult.done === true
                && !ranThrow && thrownValue === 9 && incompatibleReceiverThrows
                && reentrantThrows && afterReentrancy.done === true;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator control methods should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }

    #[test]
    fn wasm_backend_object_and_class_generator_methods_preserve_this_and_prototypes() {
        let source = r#"
            const object = {
                value: 3,
                *method() { return this.value; }
            };
            class Example {
                constructor() { this.value = 4; }
                *method() { return this.value; }
            }
            const objectIterator = object.method();
            const instance = new Example();
            const classIterator = instance.method();
            objectIterator.next().value === 3
                && classIterator.next().value === 4
                && Object.getPrototypeOf(objectIterator) === object.method.prototype
                && Object.getPrototypeOf(classIterator) === instance.method.prototype;
        "#;
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("generator methods should run: {err:?}"));
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }
}
