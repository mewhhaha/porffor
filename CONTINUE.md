# CONTINUE — handoff state (2026-10-09)

Work directly on `main` and push to `origin/main` (no feature branches).
The task plan lives in `tasks/` (start with `tasks/README.md`); this file is
the short operational handoff for the next session.

## What landed in this round

- **Runtime/program module split.** Heap-using programs compile to a small
  program module P linked against one process-wide runtime module R
  (`crates/lila-aot-wasm/src/runtime_artifact.rs`, `runtime_artifact/cache.rs`,
  `function_layout.rs`, `program_hooks.rs`; engine side
  `crates/lila-engine/src/wasm_runtime_link.rs`). Every builtin is always
  compiled, host imports are a fixed superset, R is byte-identical for every
  program and is cached on disk. Per-program emit/compile dropped from
  ~40–60 s to a few seconds.
- **Builtins install exactly once per realm.** `RuntimeBootstrapPlan::full()`
  installs every standard builtin; host builtins install according to the host
  surface, never "because compiled". The Uint8Array base64/hex methods were
  compiled but never installed; they now have an installer
  (`StandardBuiltinInstaller::Uint8Array`). A catalog-wide audit found no other
  uninstalled builtin.
- **Semantic fixes.** Template substitutions use ToString (hint String);
  direct-eval detection inside methods; awaited `var` patterns declare their
  names (`export var x = await …`); module-syntax stripper handles string
  export names and `with {}`; dependency-module syntax errors are
  resolution-phase (`RejectedInDependency`); super/parenthesized destructuring
  targets (`DestructuringTargetIr::AssignmentSuper`); static builtin
  "requires …" restrictions removed; Annex B labelled block functions;
  `with`-environment PutValue evaluates the RHS once (was copied into every
  branch); Intl UTF-16 host-wire framing (byte lengths) fixed.
- **Runtime performance.** Newest-first own-property lookup with a
  reference-identity key fast path; `push` no longer snapshots/sorts indices
  when the array grows; private fields install through one helper call
  (6,000-field classes compile); private-name tables built without quadratic
  GC live sets.
- **Harness.** Typed Test262 failure origins (`wasm-backend` added, no message
  substring guessing); Test262 host declares a complete embedded module catalog
  so computed `import()` resolves (`crates/lila-test262/src/module_catalog.rs`).
- **Test layout.** Integration tests are consolidated by area:
  `lila-aot-wasm` 219 → 7 targets, `lila-engine` 365 → 11 targets
  (`crates/<crate>/tests/<area>/main.rs`). Use `--test <area> <module>::` to
  select. This is what keeps `target/debug` from filling the disk.

## Verification status at handoff

| Scope | Result |
|---|---|
| `cargo xc --all-features` (workspace, all targets) | passes at this commit |
| `lila-ir`, `lila-intl`, `lila-front`, `lila-runtime` | 3,102 / 3,102 pass (lila-ir alone 2,163 after the `with` fix) |
| `lila-test262` | all pass (default features) |
| `lila-aot-wasm` (lib + 7 targets) | 327 lib + 641 integration, all pass |
| `lila-engine` full suite | **interrupted** at 2,339 passed / 96 failed (not finished) |
| `lila-cli` suite, fake suite, Test262 replay | **not run** this round |

### Known `lila-engine` failures (interrupted run, pre-Uint8Array fix)

The 28 Uint8Array codec failures are **fixed** since that run. Remaining,
grouped (test module = file under `crates/lila-engine/tests/<area>/`):

- Generators / async generators (two lanes were mid-investigation, no fix
  landed): `aot_generator_for_of_continuations` (7), `aot_gc_generator_entries`
  (4), `aot_generator_{object_literal,object_patterns,optional_regions,reference_operands}`,
  `aot_async_{for_in,with}`, `aot_async_generator_{for_in_lifecycle,for_in_regions,switch_regions,pattern_regions,expression_regions}`
  (`unbound identifier received`; a `#value` parse error — check whether the
  fixture is valid JS), `aot_optional_property_await`, `aot_logical_assignment_await` (2).
- Realm / intrinsic identity and "poisoned builtin must be re-read" tests:
  `aot_indexed_collection_invocation`, `aot_invocation_shortcut_retirement` (2),
  `aot_number_string_hook_invocation`, `aot_numeric_native_caller_effects`,
  `aot_string_invocation_family`, `aot_json_stringify_preparation`,
  `aot_json_canonical_reviver`, `aot_global_error_caller_effects`,
  `aot_remaining_invocation_references`, `aot_ordinary_global_assignment_reference`,
  `aot_fresh_script_global_lexicals`. Suspect compile-time facts that assumed a
  script-derived bootstrap plan (now every builtin is installed).
- Entry/GC tests: `aot_gc_{collection,iterator,string_regexp,typed_array_immutable_properties,typed_array_method,binary_data}_entries`,
  `aot_tagged_template_source_owners`, `aot_callable_capture_lifecycle`,
  `aot_object_binding_single_get`.
- Likely stale after the R/P split (artifact inspection): `aot_intl_compilation_profile`
  (16, Intl sections now live in R), `tests::wasm_backend_*` (2, expect the
  old single-module export list), `runtime_regexp_compiler_tests` (1, exact
  byte image), `aot_native_function_source_syntax` (2), `aot_module_import_jobs` (2),
  `aot_embedded_module_graph` (1).
- `aot_float16_array`: execution timeout (performance).

## Next steps (in order)

1. Finish the `lila-engine` suite and fix the groups above (one implementer
   lane per group works well; give each its own `CARGO_TARGET_DIR` when a full
   suite is running).
2. Run the `lila-cli` suite (`cargo test -p lila-cli --test cli`) and the fake
   Test262 suite; fix fallout from the R/P split.
3. Replay the 5,365 historical Test262 failures with a fresh release build
   (exact `mode:path.js` ids; the 2026-09-30 aggregate is under
   `target/publication-freeze-20260930-*/test262/snapshots/`), group what
   remains into families and fix. Earlier sampling: ~84 % of non-Intl and
   ~96 % of Intl historical failures already pass; runtime-string
   `eval`/`new Function` remain excluded by design.
4. Remaining performance: amortized property-table growth (every add copies
   the table today; `objects.rs` `emit_ordinary_append_property_entry_inner`),
   doubling growth for indexed tables (`objects/define_property.rs`
   `emit_grow_indexed_table`), a hash index for large objects such as the
   global object, and folding HasProperty+Get for global reads.
5. Small known gaps: vendored `boa_parser` rejects `[(a.b)] = x`
   (parenthesized member target without default); `import … with { type: "bytes" }`
   needs a Bytes module kind; logical/compound `with` assignments still copy the
   RHS per branch; `lila-ir/src/modules/source.rs` strips module syntax by text
   scanning (fragile — derive from the AST).

## Operational notes

- Run heavy work in memory-capped systemd scopes inside `lila-verify.slice`
  (`systemd-run --user --scope --slice=lila-verify.slice -p MemoryMax=… -p MemorySwapMax=0`);
  the user runs other heavy apps on this machine.
- Check `df -h /home` before broad suites. `target/debug` grew to 436 GB once
  and filled the disk; `cargo clean --profile dev --offline` is safe
  (first rebuild ~10–15 min).
- Use a private `LILA_CACHE_DIR` per compiler build for Test262 replays;
  concurrent builds sharing `~/.cache/lila` evict each other.
