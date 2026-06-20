import fs from 'node:fs';
import path from 'node:path';

const defaultOutput = 'worker-configuration.d.ts';
const defaultConfigNames = [
  'wrangler.jsonc',
  'wrangler.json',
  'wrangler.toml',
  'porffor.jsonc',
  'porffor.json',
  'porffor.toml'
];

const bindingSpecs = [
  [ [ 'kv_namespaces' ], 'KVNamespace' ],
  [ [ 'r2_buckets' ], 'R2Bucket' ],
  [ [ 'd1_databases' ], 'D1Database' ],
  [ [ 'durable_objects', 'bindings' ], 'DurableObjectNamespace' ],
  [ [ 'services' ], 'Fetcher' ],
  [ [ 'queues', 'producers' ], 'Queue' ],
  [ [ 'analytics_engine_datasets' ], 'AnalyticsEngineDataset' ],
  [ [ 'vectorize' ], 'VectorizeIndex' ],
  [ [ 'ai_search' ], 'AiSearch' ],
  [ [ 'ai_search_namespaces' ], 'AiSearchNamespace' ],
  [ [ 'mtls_certificates' ], 'Fetcher' ],
  [ [ 'browser' ], 'BrowserRendering' ],
  [ [ 'images' ], 'ImagesBinding' ],
  [ [ 'hyperdrive' ], 'Hyperdrive' ],
  [ [ 'workflows' ], 'Workflow' ],
  [ [ 'pipelines' ], 'Pipeline' ],
  [ [ 'dispatch_namespaces' ], 'DispatchNamespace' ],
  [ [ 'send_email' ], 'SendEmail' ]
];

const singletonBindingSpecs = [
  [ [ 'ai' ], 'Ai' ],
  [ [ 'version_metadata' ], 'WorkerVersionMetadata' ],
  [ [ 'assets' ], 'Fetcher' ]
];

const knownHandlers = [
  'fetch',
  'scheduled',
  'queue',
  'email',
  'tail',
  'trace',
  'alarm'
];

const typegenHelp = () => {
  console.log(`Usage: \x1B[1mporf types [entrypoint] [worker-configuration.d.ts] [options]\x1B[0m`);
  console.log();
  console.log('Generate Wrangler-style TypeScript runtime and Env declarations from a worker config.');
  console.log();
  console.log('Options:');
  console.log('  --config, -c <path>          Path to wrangler/porffor config; can be repeated');
  console.log('  --entrypoint <path>          Worker entrypoint when not set by config main');
  console.log('  --env, -e <name>             Generate only one named environment');
  console.log('  --env-interface <name>       Global env interface name (default: Env)');
  console.log('  --include-runtime=<bool>     Include minimal runtime declarations (default: true)');
  console.log('  --include-env=<bool>         Include env declarations (default: true)');
  console.log('  --strict-vars=<bool>         Preserve literal var types (default: true)');
  console.log('  --check                      Exit non-zero if the output file is stale');
  console.log('  --print                      Print generated declarations instead of writing');
  console.log('  --cwd <path>                 Resolve config and output paths from this directory');
};

const isFlag = x => x.startsWith('-');

const boolFromString = (value, fallback = true) => {
  if (value == null) return fallback;

  switch (String(value).toLowerCase()) {
    case '1':
    case 'true':
    case 'yes':
    case 'on':
      return true;
    case '0':
    case 'false':
    case 'no':
    case 'off':
      return false;
    default:
      throw new Error(`Expected a boolean value, got ${value}`);
  }
};

const parseArgs = args => {
  const options = {
    configPaths: [],
    cwd: process.cwd(),
    env: undefined,
    envInterface: 'Env',
    includeRuntime: true,
    includeEnv: true,
    strictVars: true,
    check: false,
    print: false,
    entrypoint: undefined,
    entrypointExplicit: false,
    output: undefined,
    help: false
  };

  const positionals = [];

  const readValue = (arg, i, flag) => {
    const eq = arg.indexOf('=');
    if (eq !== -1) return [ arg.slice(eq + 1), i ];
    if (i + 1 >= args.length || isFlag(args[i + 1])) throw new Error(`${flag} expects a value`);
    return [ args[i + 1], i + 1 ];
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (!isFlag(arg)) {
      positionals.push(arg);
      continue;
    }

    const [ flagName, inlineValue ] = arg.split('=', 2);
    switch (flagName) {
      case '--help':
      case '-h':
        options.help = true;
        break;
      case '--config':
      case '-c': {
        const [ value, next ] = readValue(arg, i, flagName);
        options.configPaths.push(value);
        i = next;
        break;
      }
      case '--cwd': {
        const [ value, next ] = readValue(arg, i, flagName);
        options.cwd = path.resolve(value);
        i = next;
        break;
      }
      case '--env':
      case '-e': {
        const [ value, next ] = readValue(arg, i, flagName);
        options.env = value;
        i = next;
        break;
      }
      case '--env-interface': {
        const [ value, next ] = readValue(arg, i, flagName);
        if (!/^[A-Za-z_$][\w$]*$/.test(value)) throw new Error(`Invalid TypeScript interface name: ${value}`);
        options.envInterface = value;
        i = next;
        break;
      }
      case '--entrypoint':
      case '--entry':
      case '--main': {
        const [ value, next ] = readValue(arg, i, flagName);
        options.entrypoint = value;
        options.entrypointExplicit = true;
        i = next;
        break;
      }
      case '--out':
      case '--output':
      case '-o': {
        const [ value, next ] = readValue(arg, i, flagName);
        options.output = value;
        i = next;
        break;
      }
      case '--include-runtime':
        options.includeRuntime = boolFromString(inlineValue, true);
        break;
      case '--no-include-runtime':
        options.includeRuntime = false;
        break;
      case '--include-env':
        options.includeEnv = boolFromString(inlineValue, true);
        break;
      case '--no-include-env':
        options.includeEnv = false;
        break;
      case '--strict-vars':
        options.strictVars = boolFromString(inlineValue, true);
        break;
      case '--no-strict-vars':
        options.strictVars = false;
        break;
      case '--check':
        options.check = boolFromString(inlineValue, true);
        break;
      case '--print':
        options.print = boolFromString(inlineValue, true);
        break;
      default:
        throw new Error(`Unknown types option: ${flagName}`);
    }
  }

  for (const positional of positionals) {
    if (positional.endsWith('.d.ts')) {
      if (options.output) throw new Error(`Multiple output paths were provided: ${options.output}, ${positional}`);
      options.output = positional;
      continue;
    }

    if (!options.entrypoint) {
      options.entrypoint = positional;
      options.entrypointExplicit = true;
      continue;
    }

    if (!options.output) {
      options.output = positional;
      continue;
    }

    throw new Error(`Unexpected positional argument: ${positional}`);
  }

  options.output ??= defaultOutput;
  if (!options.output.endsWith('.d.ts')) throw new Error(`Type output path must end in .d.ts: ${options.output}`);

  return options;
};

const resolveFrom = (base, value) => path.isAbsolute(value) ? value : path.resolve(base, value);

const discoverConfig = cwd => {
  for (const name of defaultConfigNames) {
    const candidate = path.join(cwd, name);
    if (fs.existsSync(candidate)) return [ candidate ];
  }

  return [];
};

const stripJsonComments = source => {
  let out = '';
  let inString = false;
  let quote = '';
  let escaped = false;

  for (let i = 0; i < source.length; i++) {
    const ch = source[i];
    const next = source[i + 1];

    if (inString) {
      out += ch;
      if (escaped) {
        escaped = false;
      } else if (ch === '\\') {
        escaped = true;
      } else if (ch === quote) {
        inString = false;
      }
      continue;
    }

    if (ch === '"' || ch === "'") {
      inString = true;
      quote = ch;
      out += ch;
      continue;
    }

    if (ch === '/' && next === '/') {
      while (i < source.length && source[i] !== '\n') i++;
      out += '\n';
      continue;
    }

    if (ch === '/' && next === '*') {
      i += 2;
      while (i < source.length && !(source[i] === '*' && source[i + 1] === '/')) i++;
      i++;
      continue;
    }

    out += ch;
  }

  return out;
};

const stripTrailingJsonCommas = source => {
  let out = '';
  let inString = false;
  let quote = '';
  let escaped = false;

  for (let i = 0; i < source.length; i++) {
    const ch = source[i];

    if (inString) {
      out += ch;
      if (escaped) {
        escaped = false;
      } else if (ch === '\\') {
        escaped = true;
      } else if (ch === quote) {
        inString = false;
      }
      continue;
    }

    if (ch === '"' || ch === "'") {
      inString = true;
      quote = ch;
      out += ch;
      continue;
    }

    if (ch === ',') {
      let j = i + 1;
      while (/\s/.test(source[j] ?? '')) j++;
      if (source[j] === '}' || source[j] === ']') continue;
    }

    out += ch;
  }

  return out;
};

const parseJsonLike = (source, filePath) => {
  try {
    return JSON.parse(stripTrailingJsonCommas(stripJsonComments(source)));
  } catch (e) {
    throw new Error(`Failed to parse ${filePath}: ${e.message}`);
  }
};

const stripTomlComment = line => {
  let inString = false;
  let quote = '';
  let escaped = false;

  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (inString) {
      if (escaped) {
        escaped = false;
      } else if (ch === '\\') {
        escaped = true;
      } else if (ch === quote) {
        inString = false;
      }
      continue;
    }

    if (ch === '"' || ch === "'") {
      inString = true;
      quote = ch;
      continue;
    }

    if (ch === '#') return line.slice(0, i);
  }

  return line;
};

const splitToml = (value, delimiter = ',') => {
  const parts = [];
  let current = '';
  let depth = 0;
  let inString = false;
  let quote = '';
  let escaped = false;

  for (const ch of value) {
    if (inString) {
      current += ch;
      if (escaped) {
        escaped = false;
      } else if (ch === '\\') {
        escaped = true;
      } else if (ch === quote) {
        inString = false;
      }
      continue;
    }

    if (ch === '"' || ch === "'") {
      inString = true;
      quote = ch;
      current += ch;
      continue;
    }

    if (ch === '[' || ch === '{') depth++;
    if (ch === ']' || ch === '}') depth--;

    if (ch === delimiter && depth === 0) {
      parts.push(current.trim());
      current = '';
    } else {
      current += ch;
    }
  }

  if (current.trim() || value.endsWith(delimiter)) parts.push(current.trim());
  return parts;
};

const splitTomlPath = value => splitToml(value, '.').map(x => x.replace(/^["']|["']$/g, ''));

const parseTomlValue = raw => {
  const value = raw.trim();
  if ((value.startsWith('"') && value.endsWith('"')) || (value.startsWith("'") && value.endsWith("'"))) {
    const unquoted = value.slice(1, -1);
    return value[0] === '"' ? JSON.parse(value) : unquoted;
  }

  if (value === 'true') return true;
  if (value === 'false') return false;
  if (/^[+-]?\d+(\.\d+)?$/.test(value)) return Number(value);

  if (value.startsWith('[') && value.endsWith(']')) {
    const inner = value.slice(1, -1).trim();
    if (!inner) return [];
    return splitToml(inner).map(parseTomlValue);
  }

  if (value.startsWith('{') && value.endsWith('}')) {
    const out = {};
    const inner = value.slice(1, -1).trim();
    if (!inner) return out;

    for (const part of splitToml(inner)) {
      const eq = part.indexOf('=');
      if (eq === -1) throw new Error(`Invalid inline TOML table entry: ${part}`);
      setNested(out, splitTomlPath(part.slice(0, eq).trim()), parseTomlValue(part.slice(eq + 1)));
    }

    return out;
  }

  return value;
};

const getNested = (obj, parts) => parts.reduce((acc, part) => acc?.[part], obj);

function setNested(obj, parts, value) {
  let cursor = obj;
  for (const part of parts.slice(0, -1)) {
    cursor = cursor[part] ??= {};
  }
  cursor[parts.at(-1)] = value;
}

const ensureNested = (obj, parts) => {
  let cursor = obj;
  for (const part of parts) {
    cursor = cursor[part] ??= {};
  }
  return cursor;
};

const parseToml = (source, filePath) => {
  const root = {};
  let current = root;

  try {
    for (const rawLine of source.split(/\r?\n/)) {
      const line = stripTomlComment(rawLine).trim();
      if (!line) continue;

      if (line.startsWith('[[') && line.endsWith(']]')) {
        const parts = splitTomlPath(line.slice(2, -2).trim());
        const parent = ensureNested(root, parts.slice(0, -1));
        const key = parts.at(-1);
        parent[key] ??= [];
        const item = {};
        parent[key].push(item);
        current = item;
        continue;
      }

      if (line.startsWith('[') && line.endsWith(']')) {
        current = ensureNested(root, splitTomlPath(line.slice(1, -1).trim()));
        continue;
      }

      const eq = line.indexOf('=');
      if (eq === -1) throw new Error(`Invalid TOML line: ${line}`);
      setNested(current, splitTomlPath(line.slice(0, eq).trim()), parseTomlValue(line.slice(eq + 1)));
    }
  } catch (e) {
    throw new Error(`Failed to parse ${filePath}: ${e.message}`);
  }

  return root;
};

const readConfig = filePath => {
  const source = fs.readFileSync(filePath, 'utf8');
  if (filePath.endsWith('.toml')) return parseToml(source, filePath);
  return parseJsonLike(source, filePath);
};

const cloneConfigValue = value => {
  if (Array.isArray(value)) return value.map(cloneConfigValue);
  if (value && typeof value === 'object') return Object.fromEntries(Object.entries(value).map(([ key, item ]) => [ key, cloneConfigValue(item) ]));
  return value;
};

const withoutEnv = config => {
  const { env: _env, ...base } = config ?? {};
  return cloneConfigValue(base);
};

const mergeConfig = (base, override) => {
  const out = cloneConfigValue(base ?? {});
  for (const [ key, value ] of Object.entries(override ?? {})) {
    if (value && typeof value === 'object' && !Array.isArray(value) && out[key] && typeof out[key] === 'object' && !Array.isArray(out[key])) {
      out[key] = mergeConfig(out[key], value);
    } else {
      out[key] = cloneConfigValue(value);
    }
  }

  return out;
};

const isArray = value => Array.isArray(value) ? value : value == null ? [] : [ value ];

const propName = name => /^[A-Za-z_$][\w$]*$/.test(name) ? name : JSON.stringify(name);

const literalType = value => {
  if (typeof value === 'string') return JSON.stringify(value);
  if (typeof value === 'number' && Number.isFinite(value)) return String(value);
  if (typeof value === 'boolean') return String(value);
  if (value == null) return 'null';
  if (Array.isArray(value)) {
    const types = [ ...new Set(value.map(literalType)) ];
    return types.length === 0 ? 'readonly unknown[]' : `readonly (${types.join(' | ')})[]`;
  }

  return 'unknown';
};

const looseVarType = value => {
  if (typeof value === 'string') return 'string';
  if (typeof value === 'number') return 'number';
  if (typeof value === 'boolean') return 'boolean';
  if (value == null) return 'null';
  if (Array.isArray(value)) return 'readonly unknown[]';
  return 'Record<string, unknown>';
};

const addBinding = (bindings, name, type, optional = false, source = 'config') => {
  if (!name) return;

  const existing = bindings.get(name) ?? {
    name,
    types: new Set(),
    optional: true,
    sources: new Set()
  };

  existing.types.add(type);
  existing.optional &&= optional;
  existing.sources.add(source);
  bindings.set(name, existing);
};

const extractBindingsFromConfig = (config, bindings, { optional = false, source = 'config', strictVars = true } = {}) => {
  const vars = config?.vars;
  if (vars && typeof vars === 'object' && !Array.isArray(vars)) {
    for (const [ name, value ] of Object.entries(vars)) {
      addBinding(bindings, name, strictVars ? literalType(value) : looseVarType(value), optional, `${source}:vars`);
    }
  }

  for (const name of isArray(config?.secrets?.required)) {
    addBinding(bindings, name, 'string', optional, `${source}:secrets`);
  }

  for (const [ parts, type ] of bindingSpecs) {
    for (const item of isArray(getNested(config, parts))) {
      addBinding(bindings, item?.binding ?? item?.name, type, optional, `${source}:${parts.join('.')}`);
    }
  }

  for (const [ parts, type ] of singletonBindingSpecs) {
    const item = getNested(config, parts);
    addBinding(bindings, item?.binding ?? item?.name, type, optional, `${source}:${parts.join('.')}`);
  }

  for (const item of isArray(getNested(config, [ 'unsafe', 'bindings' ]))) {
    addBinding(bindings, item?.name ?? item?.binding, 'unknown', optional, `${source}:unsafe.bindings`);
  }
};

const collectBindings = (configs, options) => {
  const bindings = new Map();

  for (const config of configs) {
    const base = withoutEnv(config);
    if (options.env) {
      const envConfig = config?.env?.[options.env];
      extractBindingsFromConfig(mergeConfig(base, envConfig), bindings, {
        optional: false,
        source: options.env ? `env.${options.env}` : 'config',
        strictVars: options.strictVars
      });
      continue;
    }

    extractBindingsFromConfig(base, bindings, {
      optional: false,
      source: 'config',
      strictVars: options.strictVars
    });

    for (const [ envName, envConfig ] of Object.entries(config?.env ?? {})) {
      extractBindingsFromConfig(envConfig, bindings, {
        optional: true,
        source: `env.${envName}`,
        strictVars: options.strictVars
      });
    }
  }

  return [ ...bindings.values() ].sort((a, b) => a.name.localeCompare(b.name));
};

const readEntrypoint = (entrypoint, baseDir, explicit) => {
  if (!entrypoint) return undefined;

  const resolved = resolveFrom(baseDir, entrypoint);
  if (!fs.existsSync(resolved)) {
    if (explicit) throw new Error(`Entrypoint does not exist: ${entrypoint}`);
    return { path: entrypoint, resolved, syntax: 'unknown', handlers: [] };
  }

  const source = fs.readFileSync(resolved, 'utf8');
  const serviceWorker = /\baddEventListener\s*\(\s*['"](?:fetch|scheduled|queue|email|tail)['"]/.test(source);
  const hasDefaultExport = /\bexport\s+default\b/.test(source);
  const handlers = knownHandlers.filter(handler => new RegExp(`\\b${handler}\\s*(?:\\(|:)`).test(source));

  return {
    path: entrypoint,
    resolved,
    syntax: serviceWorker ? 'service-worker' : hasDefaultExport ? 'module' : 'unknown',
    handlers
  };
};

const relativeForComment = (cwd, filePath) => {
  const rel = path.relative(cwd, filePath);
  return rel && !rel.startsWith('..') ? rel : filePath;
};

const formatBindingType = binding => {
  const types = [ ...binding.types ].sort();
  return types.length === 1 ? types[0] : types.join(' | ');
};

const runtimeDeclarations = () => `declare interface ExecutionContext {
  waitUntil(promise: Promise<unknown>): void;
  passThroughOnException(): void;
}

declare interface ScheduledController {
  readonly scheduledTime: number;
  readonly cron: string;
  noRetry(): void;
}

declare interface Message<Body = unknown> {
  readonly id: string;
  readonly timestamp: Date;
  readonly body: Body;
  ack(): void;
  retry(options?: QueueRetryOptions): void;
}

declare interface MessageBatch<Body = unknown> {
  readonly queue: string;
  readonly messages: readonly Message<Body>[];
  ackAll(): void;
  retryAll(options?: QueueRetryOptions): void;
}

declare interface QueueRetryOptions {
  delaySeconds?: number;
}

declare interface ExportedHandler<Env = unknown> {
  fetch?(request: Request, env: Env, ctx: ExecutionContext): Response | Promise<Response>;
  scheduled?(controller: ScheduledController, env: Env, ctx: ExecutionContext): void | Promise<void>;
  queue?(batch: MessageBatch, env: Env, ctx: ExecutionContext): void | Promise<void>;
  email?(message: unknown, env: Env, ctx: ExecutionContext): void | Promise<void>;
  tail?(events: readonly unknown[], env: Env, ctx: ExecutionContext): void | Promise<void>;
  trace?(traces: readonly unknown[], env: Env, ctx: ExecutionContext): void | Promise<void>;
  alarm?(controller: unknown, env: Env, ctx: ExecutionContext): void | Promise<void>;
}

declare interface Fetcher {
  fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response>;
}

declare interface KVNamespace {
  get(key: string, options?: unknown): Promise<unknown>;
  put(key: string, value: unknown, options?: unknown): Promise<void>;
  delete(key: string): Promise<void>;
  list(options?: unknown): Promise<unknown>;
}

declare interface R2Bucket {
  get(key: string, options?: unknown): Promise<unknown>;
  put(key: string, value: unknown, options?: unknown): Promise<unknown>;
  delete(keys: string | readonly string[]): Promise<void>;
  list(options?: unknown): Promise<unknown>;
}

declare interface D1PreparedStatement {
  bind(...values: unknown[]): D1PreparedStatement;
  first<T = unknown>(columnName?: string): Promise<T | null>;
  run<T = unknown>(): Promise<T>;
  all<T = unknown>(): Promise<T>;
  raw<T = unknown>(): Promise<T[]>;
}

declare interface D1Database {
  prepare(query: string): D1PreparedStatement;
  batch<T = unknown>(statements: readonly D1PreparedStatement[]): Promise<T[]>;
  exec(query: string): Promise<unknown>;
  dump(): Promise<ArrayBuffer>;
}

declare interface DurableObjectId {
  readonly name?: string;
  toString(): string;
}

declare interface DurableObjectStub {
  fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response>;
}

declare interface DurableObjectNamespace {
  newUniqueId(options?: unknown): DurableObjectId;
  idFromName(name: string): DurableObjectId;
  idFromString(id: string): DurableObjectId;
  get(id: DurableObjectId): DurableObjectStub;
}

declare interface Queue<Body = unknown> {
  send(message: Body, options?: unknown): Promise<void>;
  sendBatch(messages: readonly { body: Body; options?: unknown }[]): Promise<void>;
}

declare interface AnalyticsEngineDataset {
  writeDataPoint(event: unknown): void;
}

declare interface VectorizeIndex {
  query(vector: readonly number[], options?: unknown): Promise<unknown>;
  insert(vectors: readonly unknown[]): Promise<unknown>;
  upsert(vectors: readonly unknown[]): Promise<unknown>;
  deleteByIds(ids: readonly string[]): Promise<unknown>;
}

declare interface Ai {
  run(model: string, inputs: unknown, options?: unknown): Promise<unknown>;
}

declare interface AiSearch {
  search(query: unknown, options?: unknown): Promise<unknown>;
}

declare interface AiSearchNamespace {
  search(query: unknown, options?: unknown): Promise<unknown>;
}

declare interface BrowserRendering {
  launch(options?: unknown): Promise<unknown>;
}

declare interface ImagesBinding {
  input(value: unknown): unknown;
}

declare interface Hyperdrive {
  readonly connectionString: string;
}

declare interface Workflow {
  create(options?: unknown): Promise<unknown>;
  get(id: string): Promise<unknown>;
}

declare interface Pipeline {
  send(records: readonly unknown[]): Promise<void>;
}

declare interface DispatchNamespace {
  get(name: string, args?: unknown): Fetcher;
}

declare interface SendEmail {
  send(message: unknown): Promise<void>;
}
`;

const generateDeclarations = ({ configs, configPaths, entrypointInfo, bindings, options }) => {
  const lines = [];
  const configList = configPaths.length > 0 ? configPaths.map(x => relativeForComment(options.cwd, x)).join(', ') : 'none';
  const entrypointLabel = entrypointInfo?.path ?? 'none';

  lines.push('// Generated by Porffor. Regenerate with `porf types` after config changes.');
  lines.push(`// Config: ${configList}`);
  lines.push(`// Entrypoint: ${entrypointLabel}`);
  if (options.env) lines.push(`// Environment: ${options.env}`);
  if (entrypointInfo) {
    lines.push(`// Entrypoint syntax: ${entrypointInfo.syntax}`);
    if (entrypointInfo.handlers.length > 0) lines.push(`// Detected handlers: ${entrypointInfo.handlers.join(', ')}`);
  }
  lines.push('');

  if (options.includeRuntime) {
    lines.push(runtimeDeclarations().trimEnd());
    lines.push('');
  }

  if (options.includeEnv) {
    lines.push('declare namespace Porffor {');
    lines.push(`  interface ${options.envInterface} {`);
    if (bindings.length === 0) {
      lines.push('    // No bindings were found in the provided config.');
    } else {
      for (const binding of bindings) {
        lines.push(`    ${propName(binding.name)}${binding.optional ? '?' : ''}: ${formatBindingType(binding)};`);
      }
    }
    lines.push('  }');

    const compatibilityDate = configs.find(config => config?.compatibility_date)?.compatibility_date;
    const compatibilityFlags = configs.flatMap(config => isArray(config?.compatibility_flags));
    if (compatibilityDate || compatibilityFlags.length > 0 || entrypointInfo) {
      lines.push('');
      lines.push('  interface WorkerConfiguration {');
      if (compatibilityDate) lines.push(`    compatibilityDate: ${JSON.stringify(compatibilityDate)};`);
      if (compatibilityFlags.length > 0) lines.push(`    compatibilityFlags: readonly (${compatibilityFlags.map(JSON.stringify).join(' | ')})[];`);
      if (entrypointInfo) {
        lines.push(`    entrypoint: ${JSON.stringify(entrypointInfo.path)};`);
        lines.push(`    syntax: ${JSON.stringify(entrypointInfo.syntax)};`);
      }
      lines.push('  }');
    }

    lines.push('}');
    lines.push('');
    lines.push(`interface ${options.envInterface} extends Porffor.${options.envInterface} {}`);
    lines.push('');
  }

  return `${lines.join('\n').trimEnd()}\n`;
};

const loadInputs = options => {
  const configPaths = (options.configPaths.length > 0 ? options.configPaths : discoverConfig(options.cwd))
    .map(configPath => resolveFrom(options.cwd, configPath));
  const configs = configPaths.map(readConfig);
  const configDir = configPaths[0] ? path.dirname(configPaths[0]) : options.cwd;
  const configMain = configs.find(config => typeof config?.main === 'string')?.main;
  const entrypoint = options.entrypoint ?? configMain;
  const entrypointInfo = readEntrypoint(entrypoint, options.entrypoint ? options.cwd : configDir, options.entrypointExplicit);
  const bindings = collectBindings(configs, options);

  return { configs, configPaths, entrypointInfo, bindings };
};

export default async function generateTypes(args = process.argv.slice(2)) {
  const options = parseArgs(args);
  if (options.help) {
    typegenHelp();
    return;
  }

  const inputs = loadInputs(options);
  const out = generateDeclarations({ ...inputs, options });
  const outputPath = resolveFrom(options.cwd, options.output);

  if (options.print) {
    process.stdout.write(out);
    return;
  }

  if (options.check) {
    const current = fs.existsSync(outputPath) ? fs.readFileSync(outputPath, 'utf8') : '';
    if (current !== out) {
      throw new Error(`${options.output} is out of date; run porf types`);
    }

    console.log(`Types are up to date at ${options.output}`);
    return;
  }

  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, out);
  console.log(`Types written to ${options.output}`);
}
