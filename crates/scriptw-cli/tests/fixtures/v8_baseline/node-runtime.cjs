// Opt-in reference harness only; this file does not implement the compiler.
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const crypto = require('node:crypto');
const [root, sampleArg, warmupArg] = process.argv.slice(2);
const samples = Number(sampleArg), warmups = Number(warmupArg);
if (!root || !Number.isInteger(samples) || samples < 3 || samples > 100 ||
    !Number.isInteger(warmups) || warmups < 1 || warmups > 1000) {
  throw new Error('usage: v8-runtime-baseline.cjs FIXTURES SAMPLES WARMUPS');
}
const plan = JSON.parse(fs.readFileSync(path.join(root, 'manifest.json'), 'utf8'));
const cases = [];
for (const entry of plan.execution) {
  if (path.basename(entry.file) !== entry.file) throw new Error('fixture must be a basename');
  const source = fs.readFileSync(path.join(root, entry.file), 'utf8');
  const script = new vm.Script(source, {filename: entry.file});
  const observations = [];
  for (let i = 0; i < warmups + samples; i++) {
    let stdout = '';
    // Fresh Realm, precompiled source, matching Wasmtime's fresh Store runs.
    const context = vm.createContext({print: line => { stdout += String(line) + '\n'; }});
    const begin = process.hrtime.bigint();
    script.runInContext(context);
    const execution_ns = Number(process.hrtime.bigint() - begin);
    if (stdout !== entry.stdout) throw new Error(entry.file + ': checksum mismatch: ' + stdout);
    observations.push({warmup: i < warmups, execution_ns, stdout});
  }
  cases.push({file: entry.file, source_sha256: crypto.createHash('sha256').update(source).digest('hex'), observations});
}
console.log(JSON.stringify({schema: 'scriptw-v8-baseline-v1', node: process.version,
  v8: process.versions.v8, samples, warmups, cases}));
