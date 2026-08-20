# Lila identity migration contract

T29 replaces the remaining Porffor-shaped Rust identity with Lila in one
coordinated change. This document and
[`lila-identity-map.tsv`](./lila-identity-map.tsv) are the source of truth for
that change. The inventory was taken from
`7ac4ee8a80e4e58b3dfb1adfece974f9f0a19e27` on 2026-08-11.

The map is intentionally written before any rename. A textual replacement is
not sufficient: Cargo identities, generated JavaScript helpers, the emitted
Wasm host ABI, caches and persisted conformance evidence have different safety
requirements.

## Canonical identity

The executable is `lila`. Workspace packages retain role suffixes so their
purpose is visible and the public library does not collide with the binary or a
possible future umbrella package:

| Current package | Canonical package | Rust crate import |
| --- | --- | --- |
| `porffor-front` | `lila-front` | `lila_front` |
| `porffor-ir` | `lila-ir` | `lila_ir` |
| `porffor-runtime` | `lila-runtime` | `lila_runtime` |
| `porffor-spec-exec` | `lila-spec-exec` | `lila_spec_exec` |
| `porffor-aot-wasm` | `lila-aot-wasm` | `lila_aot_wasm` |
| `porffor-backend-c` | `lila-backend-c` | `lila_backend_c` |
| `porffor-backend-native` | `lila-backend-native` | `lila_backend_native` |
| `porffor-engine` | `lila-engine` | `lila_engine` |
| `porffor-test262` | `lila-test262` | `lila_test262` |
| `porffor-cli` | `lila-cli` | `lila_cli` |

`lila-engine` remains the public library face. Calling the package merely
`lila` would make Cargo diagnostics, package selection and release artifacts
ambiguous with the `lila` executable. `lila-aot-wasm` is explicit because
`lila-wasm` could mean the runtime, target or compiler backend.

The repository slug and current DNS name are external locators, not product
aliases. Until those resources are actually moved,
`https://github.com/mewhhaha/porffor` remains the Cargo repository URL and
`porffor.dev` remains the `CNAME`. Clone examples should select a Lila-named
working directory explicitly:

```sh
git clone https://github.com/mewhhaha/porffor.git lila
cd lila
```

## Complete transitional inventory

The TSV represents repeated, compiler-owned namespaces as prefixes rather than
listing every generated member. That is the stronger invariant: `__porf*`,
`$Porffor*`, `$porffor$module$*`, `porffor-*` and `porffor_*` have no member
that may escape the migration merely because it was added after this inventory.
More-specific rows precede their family row and win.

The tracked product surfaces contain the following classes.

### Cargo, filesystem and distribution

- Eleven current workspace package/directory names and their underscore-form
  Rust imports. Ten are the clean-break replacements listed above;
  `lila-intl` was created after the cutover as a new Lila-native package and
  therefore has no Porffor predecessor or synthetic TSV mapping.
- Binary target `porf`, source path `src/bin/porf.rs`, Cargo's
  `CARGO_BIN_EXE_porf`, script variables `PORF`/`PORF_BIN`, and CI artifacts
  `porf-linux-x86_64`, `porf-macos` and `porf-windows-x86_64`.
- Config discovery names `porffor.jsonc`, `porffor.json` and `porffor.toml`.
- Package selectors, paths and commands in Cargo profiles, scripts, workflows,
  task documents, Rust documentation, the README and the static site.

The canonical replacements are the corresponding `lila-*`, `lila_*`, `lila`
and `lila.{jsonc,json,toml}` names. No compatibility Cargo packages, binary
symlink, config fallback or release artifact duplicate will be added.

### Environment and build variables

Every discovered environment/build variable is a clean prefix replacement:

```text
PORFFOR_CACHE_DIR                         LILA_CACHE_DIR
PORFFOR_CACHE_LIMIT_BYTES                 LILA_CACHE_LIMIT_BYTES
PORFFOR_COMPILER_FINGERPRINT              LILA_COMPILER_FINGERPRINT
PORFFOR_CPU_PERCENT                       LILA_CPU_PERCENT
PORFFOR_EMIT_FUNCTION_BODY_BUDGET_BYTES   LILA_EMIT_FUNCTION_BODY_BUDGET_BYTES
PORFFOR_EMIT_SIZE_REPORT                  LILA_EMIT_SIZE_REPORT
PORFFOR_EMIT_SIZE_REPORT_PATH             LILA_EMIT_SIZE_REPORT_PATH
PORFFOR_FUNCTION_CACHE_LIMIT_BYTES        LILA_FUNCTION_CACHE_LIMIT_BYTES
PORFFOR_GOLDEN_OUT                        LILA_GOLDEN_OUT
PORFFOR_JOBS                              LILA_JOBS
PORFFOR_LOWER_TRACE                       LILA_LOWER_TRACE
PORFFOR_MODULE_CACHE_LIMIT_BYTES          LILA_MODULE_CACHE_LIMIT_BYTES
PORFFOR_MODULE_MEMORY_CACHE_ENTRIES       LILA_MODULE_MEMORY_CACHE_ENTRIES
PORFFOR_PROGRAM_CACHE_LIMIT_BYTES         LILA_PROGRAM_CACHE_LIMIT_BYTES
PORFFOR_TEST262_DISABLE_CASE_RUNNER       LILA_TEST262_DISABLE_CASE_RUNNER
PORFFOR_TEST262_FORCE_CASE_RUNNER         LILA_TEST262_FORCE_CASE_RUNNER
PORFFOR_VERIFY_FUNCTION_CACHE             LILA_VERIFY_FUNCTION_CACHE
PORFFOR_WASM_DUMP                         LILA_WASM_DUMP
PORFFOR_WASM_TRACE                        LILA_WASM_TRACE
PORFFOR_WASM_TRACE_DUMP                   LILA_WASM_TRACE_DUMP
```

Old variables are ignored. Accepting both prefixes would make a conflicting
pair depend on undocumented precedence and would keep every invocation
environment ambiguous.

Seven Rust constants share that spelling but are not environment variables:
`PORFFOR_GENERATOR_THROW_SLOT`, `PORFFOR_ITERATOR_FROM_WRAPPER_SLOT`,
`PORFFOR_STATIC_GENERATOR_ITERATOR_SLOT`,
`PORFFOR_STATIC_GENERATOR_VALUES_METHOD`,
`PORFFOR_YIELD_STAR_GENERATOR_SLOT`,
`PORFFOR_YIELD_STAR_RETURN_NON_OBJECT_SLOT` and
`PORFFOR_YIELD_STAR_THROW_NON_OBJECT_SLOT`. They become the corresponding
`LILA_*` constants.

### Compiler-owned names and Wasm ABI

- All compiler and harness JavaScript helpers in the `__porf*` namespace move
  to `__lila*`. This includes host functions, assertion helpers, agent hooks,
  realm hooks, Intl state and first-party fixtures. The isolated
  `__porfforSpecBoolean` fixture name becomes `__lilaSpecBoolean` too.
- All IR-only property keys in the `$Porffor*` namespace move to `$Lila*`.
- The module-linker namespace `$porffor$module$*` moves to
  `$lila$module$*`.
- The emitted Wasm import module `porf_host` moves to `lila_host`. Emitter and
  engine linker must change together; emitted modules using the old import
  module are rejected rather than adapted.
- The Wasm name-section module `porffor` and displayed main name
  `porffor::main` become `lila` and `lila::main`.
- Host member names such as `print_line_utf8`, `number_pow`, `agent_call`,
  `memory`, `result_tag` and `completion_kind` do not carry project identity
  and remain unchanged.

There is no dual ABI. Supporting both `porf_host` and `lila_host` would permit
an old emitted module to enter a new runtime despite cache and compiler
fingerprint changes.

### Cache and process-local identity

- The default platform cache root changes from `porffor` to `lila`.
- Compiler salts `porffor-program-cache-compiler-v2` and
  `porffor-program-cache-compiler-v3` become the byte-distinct `lila-*`
  equivalents. Existing version suffixes describe their formats and are not
  reset merely for branding.
- Rust fields/functions `porffor_bytes_removed`, `porffor_files_removed` and
  `porffor_cache_root`, plus CLI labels `porffor-cache-total-*`,
  `porffor-files-removed` and `porffor-bytes-removed`, become `lila_*` and
  `lila-*`.
- Thread, temporary-directory and attempt-journal prefixes under the owned
  `porffor-*` namespace become `lila-*`.

The migration never moves or deletes the old default cache. The first Lila run
uses a cold `lila` root. `lila cache status` and `lila cache prune` inspect and
modify only that root unless the user explicitly points `LILA_CACHE_DIR`
elsewhere. The old `PORFFOR_CACHE_DIR` is ignored. Thus an old entry, a new
entry and unrelated sibling data have deterministic outcomes: old and sibling
data are untouched; only the new root is eligible for normal pruning.

Even if a user deliberately points `LILA_CACHE_DIR` at an old directory, the
renamed compiler fingerprints and program-cache salts prevent an old program
artifact from matching a new key. Wasmtime's own separately keyed stencil and
module entries remain Wasmtime's responsibility.

### Diagnostics, status and persisted Test262 data

- Current diagnostics and trace prefixes say `lila`, including
  `unsupported in lila wasm-aot`, `not supported in lila-spec-exec` and
  `lila wasm trace:`. Classifiers use typed diagnostic codes where available;
  any remaining string classifier is updated atomically with its producer.
- README generated-block markers become `lila-status:start` and
  `lila-status:end`. The publisher and repository check change in the same
  patch. Marker replacement must preserve the block body until a verified
  publisher run refreshes it.
- Published status filenames and existing JSON field names contain no Porffor
  identity today and remain stable. New status JSON records
  `producer: "lila"`.
- Snapshot, matrix-cache and backlog writes move from snapshot version 5 to
  version 6 and record the same closed producer identity. Attempt journals use
  their own format version, which moves from 1 to 2 and records that producer.
  Version-1 journals start fresh rather than carrying crash strikes across the
  compiler identity boundary. Only a Lila-produced verified aggregate may feed
  `publish-status`.

Tracked version-4/version-5 snapshots are evidence, not source code. Their
diagnostic strings and `detail_hash` values must not be mass-rewritten. A
typed legacy decoder may read them for comparison, triage and backlog reports,
but they may not be resumed, merged or published. This is the migration's only
bounded compatibility path. Its owner is T03/T29 and it is removed after the
first verified full pinned Lila aggregate replaces the Porffor-era evidence,
and no later than 1.0.

## Migration order

The implementation is one identity batch with dependency-ordered edits. No
intermediate commit is a supported release.

1. Add an identifier-boundary audit driven by the TSV. During the batch it
   rejects new unmapped spellings but permits the inventory itself, T28's
   historical record, immutable snapshots, the recovery commit and exact
   external locators.
2. Rename foundation directories/packages and imports in dependency order:
   `front`/`runtime`, `ir`, backends/spec executor, `engine`, `test262`, then
   `cli`. Update workspace profiles and regenerate `Cargo.lock` once.
3. In the same compiler layer, rename the closed JavaScript helper/property
   namespaces, module-linker names, Wasm name section and both sides of the
   `lila_host` ABI.
4. Rename the binary, config names, environment variables, build-script output,
   cache root/salts/fields, diagnostics, thread/temp names and CLI output.
5. Bump persisted conformance data to the Lila producer/version contract;
   retain the isolated read-only legacy decoder.
6. Update scripts, workflows, release artifact names, current task commands,
   README/site/contributor documentation and generated-block tooling. Preserve
   exact external repository and DNS locators.
7. Make the boundary audit strict: no transitional token is accepted in
   product code, current commands or new artifacts. Only the documented
   historical/generated/external scopes remain.

## Boundary invariant

After cutover, a repository audit must reject these namespaces in current
product and automation surfaces:

```text
porffor-   porffor_   PORFFOR_   __porf   $Porffor
$porffor$module$     porf_host   CARGO_BIN_EXE_porf
```

It also rejects the standalone `porf` executable/config/release name and the
old diagnostic/status phrases. The allowlist is structural, not a growing list
of individual lines:

- `docs/rust-rewrite/lila-identity-*` and T29 may describe the mapping;
- T28 and the legacy-retirement section of `AGENTS.md` may name history;
- tracked version-4/version-5 snapshot payloads remain immutable;
- the exact GitHub repository URL, `CNAME`, and recovery commit remain valid;
- vendored/Test262 upstream content is outside project identity.

Any new exception needs an owner, a reason and a removal condition in T29. A
free-form textual allowlist is not acceptable.
