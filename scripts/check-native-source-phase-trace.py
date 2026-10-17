#!/usr/bin/env python3
"""Default-off diagnostic ownership; runtime timing assertions remain measurements."""
from pathlib import Path
import re
import sys

E = 'crates/lila-engine/src/'
C = E + 'wasm_native_source_compiler.rs'
T = E + 'wasm_native_source_compiler/trace.rs'
N = E + 'wasm_native_program_template.rs'
KINDS = {'function': 'Function', 'global_script': 'GlobalScript', 'direct_eval': 'DirectEval'}
FILES = (C, T, N) + tuple(E + 'wasm_native_' + k + '_compiler' + tail
    for k in KINDS for tail in ('.rs', '/factory.rs'))

def compact(source):
    return re.sub(r'\s+', '', re.sub(r'//[^\n]*', '', source))

def check(root, overrides=None):
    overrides = overrides or {}
    sources = {p: overrides[p] if p in overrides else (root / p).read_text() for p in FILES}
    c = {p: compact(source) for p, source in sources.items()}
    errors = []
    def need(ok, why):
        if not ok: errors.append(why)
    context, trace = c[C], c[T]
    need('modtrace;' in context and 'trace:Option<TraceState>' in context and
         'trace:TraceState::from_environment()' in context and
         'self.trace.as_ref()?.begin(family)' in context, 'execution owns optional trace and explicit callback handles')
    need('var_os("LILA_NATIVE_SOURCE_TRACE").map(|_|Self{origin:Instant::now(),' in trace and
         'LILA_WASM_TRACE' not in trace and trace.count('var_os(') == 1,
         'dedicated opt-in creates no Instant when disabled')
    need('constCALLBACK_LIMIT:u64=256;' in trace and 'callbacks:AtomicU64' in trace and
         '.fetch_update(Ordering::Relaxed,Ordering::Relaxed,|count|{(count<CALLBACK_LIMIT).then_some(count+1)})' in trace and
         '.ok()?;' in trace, 'execution-local counter saturates at bounded callback limit')
    need('implDropforCallbackTrace' in trace and 'implDropforPhaseSpan' in trace and
         'self.record("callback","begin",trace.start,0)' not in trace and
         'trace.record("callback","begin",trace.start,0)' in trace and
         'self.record(phase.name(),"begin",start,0)' in trace and
         'self.record("callback","end",end,' in trace and
         'self.callback.record(self.phase.name(),"end",end,' in trace,
         'callback and child spans close by RAII including error paths')
    need(trace.count('eprintln!(') == 1 and
         'v=1callback={}family={}phase={}event={}at_ns={}elapsed_ns={}' in trace and
         'source:' not in trace and 'String' not in trace and 'WasmHostState' not in trace and
         'thread_local!' not in trace and 'static' not in re.sub(r"&'staticstr", '', trace),
         'bounded numeric protocol has no source, semantic roots or ambient execution state')
    for kind, family in KINDS.items():
        host = c[E + 'wasm_native_' + kind + '_compiler.rs']
        factory = c[E + 'wasm_native_' + kind + '_compiler/factory.rs']
        need('context.begin_trace(Family::' + family + ')' in host and
             'trace.as_ref().map(|trace|trace.phase(Phase::Decode))' in host and
             'drop(decode);' in host and 'Phase::ParseLower' in host and
             host.find('Phase::ParseLower') < host.find('letlowered=lower_runtime_') < host.find('drop(parse_lower)') and
             'signature,trace.as_ref(),' in host,
             'actual callback threads its trace across decode/lower/factory: ' + kind)
        need('trace:Option<&CallbackTrace' in factory and
             all('Phase::' + p in factory for p in ('Emit', 'ArtifactChecks', 'TemplateCanonicalize', 'FactoryInvoke')) and
             factory.find('Phase::Emit') < factory.find('letartifact=lila_aot_wasm::emit_runtime_') < factory.find('drop(emit)') and
             factory.find('Phase::ArtifactChecks') < factory.find('!Arc::ptr_eq') < factory.find('drop(checks)') and
             factory.find('Phase::TemplateCanonicalize') < factory.find('lettemplate=lila_aot_wasm::NativeProgramTemplate::for_') < factory.find('drop(canonicalize)') and
             'instantiate(&mut*store,engine,context,&template,trace,)?' in factory and
             factory.find('Phase::FactoryInvoke') < factory.find('initialize.call(') < factory.find('factory.call('),
             'natural artifact/template/invoke boundaries retain actual execution: ' + kind)
    binder = c[N]
    need('trace:Option<&CallbackTrace' in binder and
         all('Phase::' + phase in binder for phase in ('CompiledModuleLookup', 'BindImports', 'Instantiate')) and
         binder.find('Phase::CompiledModuleLookup') < binder.find('cache::module(') < binder.find('drop(lookup)') and
         binder.find('Phase::BindImports') < binder.find('forimportinmodule.imports()') < binder.find('drop(bind)') and
         binder.find('Phase::Instantiate') < binder.find('Instance::new(') < binder.find('drop(instantiate)') and
         'lilanativetemplate:cache={}code_bytes={}imports={}images={}factories={}' in binder,
         'typed same-Store binder measures cache/bind/instantiate and retains original template protocol')
    need(len(sources[T].splitlines()) <= 180, 'private trace owner stays bounded')
    return errors

if __name__ == '__main__':
    failures = check(Path(sys.argv[1]) if len(sys.argv) == 2 else Path.cwd())
    for failure in failures: print('native source phase trace: ' + failure, file=sys.stderr)
    raise SystemExit(bool(failures))
