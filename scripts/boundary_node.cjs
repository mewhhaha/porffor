// Independent V8 control for the portable-boundary diagnostic cohort.
// This is benchmark instrumentation, not a product compiler/runtime.
'use strict';
const fs = require('node:fs');
const vm = require('node:vm');
const crypto = require('node:crypto');
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const now = () => process.hrtime.bigint();
const ns = duration => Number(duration);

function resultReady(stdout) {
  const fdText = process.env.SCRIPTW_BOUNDARY_READY_FD;
  const nonce = process.env.SCRIPTW_BOUNDARY_READY_NONCE;
  if (fdText === undefined && nonce === undefined) return null;
  if (!/^\d+$/.test(fdText || '') || !Number.isSafeInteger(Number(fdText)) || Number(fdText) < 3
      || !/^[0-9a-f]{32}$/.test(nonce || '')) throw new Error('invalid paired result-ready pipe/nonce');
  const fd = Number(fdText);
  if (!fs.fstatSync(fd).isFIFO()) throw new Error('result-ready descriptor is not a pipe');
  const stdout_sha256 = hash(Buffer.from(stdout));
  fs.writeSync(fd, `SWREADY1 ${nonce} ${stdout_sha256}\n`);
  return {nonce, stdout_sha256};
}

function run() {
  let [mode, sourcePath, cachePath, expectedPath, sampleText] = process.argv.slice(2);
  const count = process.argv.length - 2;
  if (!(['prepare', 'fresh'].includes(mode) && count === 4 || mode === 'calls' && count === 5
      || mode === 'source' && count === 3)) {
    throw new Error('usage: boundary_node.cjs source SOURCE EXPECTED | prepare|fresh SOURCE CACHE EXPECTED | calls SOURCE CACHE EXPECTED SAMPLES');
  }
  if (mode === 'source') { expectedPath = cachePath; cachePath = null; }
  const samples = mode === 'calls' ? Number(sampleText) : 0;
  if (mode === 'calls' && (!Number.isInteger(samples) || samples < 7 || samples > 100)) {
    throw new Error('call samples must be 7..100');
  }
  const expected = fs.readFileSync(expectedPath, 'utf8');
  const started = now();
  const sourceBytes = fs.readFileSync(sourcePath);
  const cache = ['prepare', 'source'].includes(mode) ? null : fs.readFileSync(cachePath);
  const readNs = now() - started;
  const constructorStarted = now();
  const script = new vm.Script(sourceBytes.toString('utf8'), {
    filename: sourcePath, ...(cache === null ? {} : {cachedData: cache})
  });
  const constructorNs = now() - constructorStarted;
  if (cache !== null && script.cachedDataRejected !== false) {
    throw new Error('V8 rejected precompiled cachedData; source fallback is not a valid observation');
  }
  let stdout = '';
  const callbacks = [];
  let markerError = null;
  const contextStarted = now();
  const context = vm.createContext({print(value) {
    const text = String(value);
    stdout += text + '\n';
    if (stdout.length > 1024 * 1024) throw new Error('benchmark output exceeded 1 MiB');
    if (mode === 'calls' && text.startsWith('scriptw:call:')) {
      // Scriptw captures output before dispatching HostHooks::print_line too.
      const entry = now();
      const position = callbacks.length;
      const index = Math.floor((position - 1) / 2);
      const expectedMarker = position === 0 ? 'scriptw:call:ready'
        : index < samples ? `scriptw:call:${position % 2 === 1 ? 'start' : 'end'}:${index}` : null;
      if (markerError || expectedMarker !== text) {
        markerError = `unexpected call marker at position ${position}: ${text}`;
        return;
      }
      const callback = {marker: text, entry, exit: 0n};
      callbacks.push(callback);
      callback.exit = now();
    }
  }});
  const contextNs = now() - contextStarted;
  const executionStarted = now();
  script.runInContext(context, {timeout: 60000});
  const executionNs = now() - executionStarted;
  const loadAndExecuteNs = now() - started;
  if (stdout !== expected) throw new Error(`unexpected V8 output: ${JSON.stringify(stdout)}`);
  if (markerError) throw new Error(markerError);
  let calls = null;
  if (mode === 'calls') {
    if (callbacks.length !== 1 + 2 * samples) throw new Error('incomplete call markers');
    const origin = callbacks[0].entry;
    calls = {
      samples_ns: Array.from({length: samples}, (_, i) => ns(callbacks[2 + 2 * i].entry - callbacks[1 + 2 * i].exit)),
      callbacks: callbacks.map(c => ({marker: c.marker, entry_ns: ns(c.entry - origin), exit_ns: ns(c.exit - origin)})),
      boundary: 'Recorded end of start-marker print callback to entry of end-marker callback. Includes marker return, one JS function call and end-marker argument/print entry; excludes setup, ten warmups and checksum printing. No subtraction. Wall time, not isolated CPU/allocation time.'
    };
  }
  // Readiness precedes source/cache provenance hashing and final JSON output.
  // Rejected cachedData or incorrect actual output can never emit this marker.
  const ready = mode === 'prepare' ? null : resultReady(stdout);
  const result = {
    schema: 'scriptw-v8-boundaries-child-v1', mode, node: process.version, v8: process.versions.v8,
    exec_argv: process.execArgv, source_sha256: hash(sourceBytes), source_bytes: sourceBytes.length,
    stdout, result_ready: ready, cached_data_supplied: cache !== null,
    cached_data_rejected: cache === null ? null : script.cachedDataRejected,
    source_and_cache_read_ns: ns(readNs), script_construction_ns: ns(constructorNs),
    context_ns: ns(contextNs), execution_ns: ns(executionNs), load_and_execute_ns: ns(loadAndExecuteNs),
    boundary: 'Fresh child reads exact source and V8 cachedData, verifies cache acceptance and runs in a fresh vm context. V8 may lazily compile native function code. Process launch, executable initialization and teardown require external child accounting.'
  };
  if (mode === 'prepare') {
    // Run once before producing cachedData so actual fixture function code may
    // enter the cache. Neither this execution nor its parsing is a score.
    const produced = script.createCachedData();
    fs.writeFileSync(cachePath, produced, {flag: 'wx'});
    result.cached_data_sha256 = hash(produced);
    result.cached_data_bytes = produced.length;
    result.cache_preparation = 'after one checked full invocation; not a measured execution';
  } else if (cache !== null) {
    result.cached_data_sha256 = hash(cache);
    result.cached_data_bytes = cache.length;
  }
  if (mode === 'source') result.boundary = 'Fresh ordinary-flag Node child reads source, constructs vm.Script without cachedData and executes in a fresh context; source compilation is included. READY follows checked actual output, before final source provenance hashing.';
  if (mode === 'calls') {
    result.guest_warmups = 10;
    result.calls = calls;
  }
  return result;
}

try { process.stdout.write(JSON.stringify(run()) + '\n'); }
catch (error) { console.error(error.stack || String(error)); process.exitCode = 1; }
