import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const cli = path.join(repoRoot, 'runtime/index.js');

const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'porffor-types-'));

const write = (filePath, source) => {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, source);
};

const runTypes = args => {
  const result = spawnSync(process.execPath, [ cli, ...args ], {
    cwd: repoRoot,
    encoding: 'utf8'
  });

  assert.equal(result.status, 0, result.stderr || result.stdout);
  return result.stdout;
};

try {
  const jsoncProject = path.join(tmpRoot, 'jsonc');
  write(path.join(jsoncProject, 'src/index.ts'), `
export default {
  fetch(request, env, ctx) {
    return new Response(env.MESSAGE);
  },
  scheduled() {}
};
`);
  write(path.join(jsoncProject, 'wrangler.jsonc'), `{
  // JSONC comments and trailing commas match common Wrangler config files.
  "main": "src/index.ts",
  "compatibility_date": "2026-06-19",
  "kv_namespaces": [{ "binding": "CACHE", "id": "x" }],
  "r2_buckets": [{ "binding": "ASSETS", "bucket_name": "assets" }],
  "vars": { "MESSAGE": "hello", "COUNT": 3 },
  "env": {
    "prod": {
      "vars": { "MESSAGE": "prod" },
      "queues": { "producers": [{ "binding": "JOBS", "queue": "jobs" }] }
    },
  },
}
`);

  const jsoncOut = runTypes([ 'types', '--cwd', jsoncProject, '--print' ]);
  assert.match(jsoncOut, /\/\/ Config: wrangler\.jsonc/);
  assert.match(jsoncOut, /\/\/ Entrypoint: src\/index\.ts/);
  assert.match(jsoncOut, /\/\/ Detected handlers: fetch, scheduled/);
  assert.match(jsoncOut, /CACHE: KVNamespace;/);
  assert.match(jsoncOut, /ASSETS: R2Bucket;/);
  assert.match(jsoncOut, /COUNT: 3;/);
  assert.match(jsoncOut, /MESSAGE: "hello" \| "prod";/);
  assert.match(jsoncOut, /JOBS\?: Queue;/);
  assert.match(jsoncOut, /compatibilityDate: "2026-06-19";/);

  const explicitProject = path.join(tmpRoot, 'explicit');
  write(path.join(explicitProject, 'src/worker.ts'), `
export default {
  fetch() {
    return new Response("ok");
  }
};
`);
  write(path.join(explicitProject, 'src/config-main.ts'), `
export default {
  scheduled() {}
};
`);
  write(path.join(explicitProject, 'wrangler.json'), JSON.stringify({
    main: 'src/config-main.ts',
    vars: { FLAG: true },
    services: [{ binding: 'API', service: 'api' }]
  }));

  const explicitEntrypointOut = runTypes([
    'types',
    '--cwd',
    explicitProject,
    '--config',
    'wrangler.json',
    '--entrypoint',
    'src/worker.ts',
    '--print'
  ]);
  assert.match(explicitEntrypointOut, /entrypoint: "src\/worker\.ts";/);
  assert.match(explicitEntrypointOut, /\/\/ Detected handlers: fetch/);
  assert.doesNotMatch(explicitEntrypointOut, /^\/\/ Detected handlers: .*scheduled/m);

  const output = 'types/worker-env.d.ts';
  runTypes([
    'types',
    'src/worker.ts',
    output,
    '--cwd',
    explicitProject,
    '--config',
    'wrangler.json'
  ]);
  runTypes([
    'types',
    'src/worker.ts',
    output,
    '--cwd',
    explicitProject,
    '--config',
    'wrangler.json',
    '--check'
  ]);
  const written = fs.readFileSync(path.join(explicitProject, output), 'utf8');
  assert.match(written, /FLAG: true;/);
  assert.match(written, /API: Fetcher;/);
  assert.match(written, /entrypoint: "src\/worker\.ts";/);
  assert.match(written, /syntax: "module";/);
  assert.match(written, /\/\/ Detected handlers: fetch/);
  assert.doesNotMatch(written, /^\/\/ Detected handlers: .*scheduled/m);

  const tomlProject = path.join(tmpRoot, 'toml');
  write(path.join(tomlProject, 'src/worker.ts'), `
addEventListener("fetch", event => event.respondWith(new Response("ok")));
`);
  write(path.join(tomlProject, 'wrangler.toml'), `
main = "src/worker.ts"
compatibility_date = "2026-06-19"

[vars]
MODE = "dev"

[[d1_databases]]
binding = "DB"
database_name = "main"

[env.preview.vars]
MODE = "preview"
`);

  const tomlOut = runTypes([
    'typegen',
    '--cwd',
    tomlProject,
    '--env',
    'preview',
    '--env-interface',
    'WorkerEnv',
    '--include-runtime=false',
    '--print'
  ]);
  assert.match(tomlOut, /interface WorkerEnv/);
  assert.match(tomlOut, /MODE: "preview";/);
  assert.match(tomlOut, /DB: D1Database;/);
  assert.match(tomlOut, /syntax: "service-worker";/);
  assert.doesNotMatch(tomlOut, /ExecutionContext/);
} finally {
  fs.rmSync(tmpRoot, { recursive: true, force: true });
}
