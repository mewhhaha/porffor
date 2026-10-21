// Opt-in reference harness only; this file does not implement the compiler.
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const crypto = require('node:crypto');
const [root, sampleArg, warmupArg, selected] = process.argv.slice(2);
const samples = Number(sampleArg), warmups = Number(warmupArg);
if (!root || !Number.isInteger(samples) || samples < 3 || samples > 100 ||
    !Number.isInteger(warmups) || warmups < 1 || warmups > 1000) {
  throw new Error('usage: v8-runtime-baseline.cjs FIXTURES SAMPLES WARMUPS CASE');
}
const plan = JSON.parse(fs.readFileSync(path.join(root, 'manifest.json'), 'utf8'));
const cases = [];
const entries = plan.execution.filter(entry => entry.file === selected);
if (entries.length !== 1) throw new Error('CASE must select exactly one manifest entry');
for (const entry of entries) {
  if (path.basename(entry.file) !== entry.file) throw new Error('fixture must be a basename');
  const source = fs.readFileSync(path.join(root, entry.file), 'utf8');
  const compileBegin = process.hrtime.bigint();
  const script = new vm.Script(source, {filename: entry.file});
  const compilation_ns = Number(process.hrtime.bigint() - compileBegin);
  const observations = [];
  for (let i = 0; i < warmups + samples; i++) {
    let stdout = '';
    // Fresh Realm, precompiled source, matching Wasmtime's fresh Store runs.
    const before_context = process.memoryUsage();
    const contextBegin = process.hrtime.bigint();
    const context = vm.createContext({print: line => { stdout += String(line) + '\n'; }});
    const context_ns = Number(process.hrtime.bigint() - contextBegin);
    const before = process.memoryUsage();
    const begin = process.hrtime.bigint();
    script.runInContext(context);
    const execution_ns = Number(process.hrtime.bigint() - begin);
    const after = process.memoryUsage();
    if (stdout !== entry.stdout) throw new Error(entry.file + ': checksum mismatch: ' + stdout);
    observations.push({warmup: i < warmups, execution_ns, context_ns, stdout,
      memory: {scope: 'process-wide-endpoints;natural-gc;not-peak-or-live-workload-bytes', before_context, before, after}});
  }
  cases.push({file: entry.file, source_sha256: crypto.createHash('sha256').update(source).digest('hex'), compilation_ns, observations});
}
console.log(JSON.stringify({schema: 'scriptw-v8-baseline-v2', node: process.version,
  v8: process.versions.v8, samples, warmups, cases}));
