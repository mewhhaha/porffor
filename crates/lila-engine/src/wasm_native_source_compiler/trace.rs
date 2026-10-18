//! Opt-in bounded timings, owned by one execution and independent of JS roots.
//! Callback/phase pairs use one execution-local clock, including nested calls.
//! Lookup includes cold compilation; factory invocation includes JS execution.
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

const CALLBACK_LIMIT: u64 = 256;

pub(crate) struct TraceState {
    origin: Instant,
    callbacks: AtomicU64,
}

#[derive(Clone, Copy)]
pub(crate) enum Family {
    Function,
    GlobalScript,
    DirectEval,
}

impl Family {
    fn name(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::GlobalScript => "global_script",
            Self::DirectEval => "direct_eval",
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Phase {
    Decode,
    ParseLower,
    Emit,
    ArtifactChecks,
    TemplateCanonicalize,
    CompiledModuleLookup,
    BindImports,
    Instantiate,
    FactoryInvoke,
}

impl Phase {
    fn name(self) -> &'static str {
        match self {
            Self::Decode => "decode",
            Self::ParseLower => "parse_lower",
            Self::Emit => "emit",
            Self::ArtifactChecks => "artifact_checks",
            Self::TemplateCanonicalize => "template_canonicalize",
            Self::CompiledModuleLookup => "compiled_module_lookup",
            Self::BindImports => "bind_imports",
            Self::Instantiate => "instantiate",
            Self::FactoryInvoke => "factory_invoke",
        }
    }
}

impl TraceState {
    pub(super) fn from_environment() -> Option<Self> {
        // No clock reads, counters or log formatting on the default path.
        std::env::var_os("LILA_NATIVE_SOURCE_TRACE").map(|_| Self {
            origin: Instant::now(),
            callbacks: AtomicU64::new(0),
        })
    }

    pub(super) fn begin(&self, family: Family) -> Option<CallbackTrace<'_>> {
        let previous = self
            .callbacks
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
                (count < CALLBACK_LIMIT).then_some(count + 1)
            })
            .ok()?;
        let trace = CallbackTrace {
            state: self,
            id: previous + 1,
            family,
            start: Instant::now(),
        };
        trace.record("callback", "begin", trace.start, 0);
        Some(trace)
    }
}

pub(crate) struct CallbackTrace<'a> {
    state: &'a TraceState,
    id: u64,
    family: Family,
    start: Instant,
}

impl CallbackTrace<'_> {
    pub(crate) fn compiled_module_cache(
        &self,
        module_hit: bool,
        function_delta: Option<(u64, u64)>,
    ) {
        let module = if module_hit { "hit" } else { "miss" };
        match function_delta {
            Some((hits, misses)) => eprintln!(
                "lila native source cache: v=1 callback={} family={} module_memory_cache={} function_cache_hits={} function_cache_misses={} counters_scope=process",
                self.id, self.family.name(), module, hits, misses,
            ),
            None => eprintln!(
                "lila native source cache: v=1 callback={} family={} module_memory_cache={} function_cache=unavailable counters_scope=process",
                self.id, self.family.name(), module,
            ),
        }
    }

    pub(crate) fn phase(&self, phase: Phase) -> PhaseSpan<'_> {
        let start = Instant::now();
        self.record(phase.name(), "begin", start, 0);
        PhaseSpan {
            callback: self,
            phase,
            start,
        }
    }

    fn record(&self, phase: &str, event: &str, at: Instant, elapsed_ns: u128) {
        eprintln!(
            "lila native source phase: v=1 callback={} family={} phase={} event={} at_ns={} elapsed_ns={}",
            self.id,
            self.family.name(),
            phase,
            event,
            at.duration_since(self.state.origin).as_nanos(),
            elapsed_ns,
        );
    }
}

impl Drop for CallbackTrace<'_> {
    fn drop(&mut self) {
        let end = Instant::now();
        self.record(
            "callback",
            "end",
            end,
            end.duration_since(self.start).as_nanos(),
        );
    }
}

pub(crate) struct PhaseSpan<'a> {
    callback: &'a CallbackTrace<'a>,
    phase: Phase,
    start: Instant,
}

impl Drop for PhaseSpan<'_> {
    fn drop(&mut self) {
        let end = Instant::now();
        self.callback.record(
            self.phase.name(),
            "end",
            end,
            end.duration_since(self.start).as_nanos(),
        );
    }
}
