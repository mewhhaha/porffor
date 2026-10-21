# Scriptw maintenance hills

Audit: 2026-10-21, baseline `6b3d9b449`. These 100 distinct candidates implement
the approved inventory; 500 is a ceiling, not a quota. The user has authorized all 100 hills. H001–H044 and H091 are complete
(45 total); the remaining 55 are queued in bounded implementation batches.
Investigation entries require evidence before a code change. This backlog does not supersede conformance acceptance.

## How to complete a hill

Record its change, validation, dependencies and completion commit below. Prefer
fewer implementations and ownership paths; preserve semantics, useful shared
runtime/cache behavior, original test budgets and all non-successes. Source moves
alone prove no compilation speedup. Current verification belongs in the
[project inventory](rust-rewrite/current-inventory.md), commands in the
[workflow](rust-rewrite/batch-workflow.md).

Validation codes: **D** = maintained links/claims and generated README blocks;
**T** = affected tooling controls, exact commands/inventories/receipts and the
complete tooling checkpoint; **R** = compilation plus focused/adjacent native
and semantic checks; **G** = source identity checks and regenerated semantic
payload comparison; **I** = characterization/measurement and a documented decision,
followed by the applicable checks only if implementation is justified.
Risk is initial review guidance, not a claim that the code contains a bug.

| ID | Priority | State / risk | Hill | Evidence / owner | Validation |
| --- | --- | --- | --- | --- | --- |
| H001 | First | Done; medium | Retire C/native/REPL product scaffolds | `engine facade, CLI dispatch, Cargo manifests, cohort anchor` | T + R |
| H002 | First | Done; low | Separate current workflow from dated procedures | `docs/rust-rewrite/batch-workflow.md` | D |
| H003 | First | Done; low | Give current status one authority | `current-inventory.md, CONTINUE.md, tasks/README.md` | D |
| H004 | First | Done; low | Correct maintained project URLs and inherited description | `Cargo.toml, CONTRIBUTING.md, index.html` | D |
| H005 | Next | Done; low | Document minimum sparse-checkout development inputs | `tracked .cargo/config.toml and setup guidance` | D |
| H006 | Next | Done; low | Extend maintained-document link checks | `scripts/conformance_owners.py::check_links` | D |
| H007 | Next | Done; low | Index active contracts, proposals and historical evidence | `docs/rust-rewrite/contracts` | D |
| H008 | Next | Done; low | Index vendor patches and repair their contract links | `runtime-artifact-persistence.md and vendor patch notes` | D |
| H009 | Next | Done; low | Make contribution evidence appropriate to cleanup work | `.github/pull_request_template.md` | D |
| H010 | Later | Done; low | Inspect evidence metadata without loading embedded payloads | `dated evidence JSON and archive receipts` | D |
| H011 | First | Done; medium | Make workspace runner setup readable and import-safe | `scripts/run_workspace_tests.py` | T |
| H012 | Next | Done; medium | Keep input validation active under optimized Python | `run_workspace_tests.py option assertions` | T |
| H013 | Next | Done; medium | Give planning and continuation one selected inventory | `workspace_test_artifacts.py and workspace runner` | T |
| H014 | Next | Done; medium | Share exact Cargo artifact discovery | `run_engine_regression_inventory.py and run_aot_unit_shard.py` | T |
| H015 | Next | Done; medium | Share libtest inventory and result parsing | `native verification runners` | T |
| H016 | Next | Done; medium | Share source identity hashing | `workspace runner and staged/native verification workers` | T |
| H017 | First | Done; low | Share publication environment initialization | `publish-real-status-low-ram.sh and lib driver` | T |
| H018 | Next | Done; medium | Share exact-scope lookup for partition and CPU policies | `workspace_test_schedule.py validators` | T |
| H019 | Next | Done; medium | Separate cohort planning and retirement admission from unlinking | `run_workspace_cohorts.py` | T |
| H020 | Next | Done; medium | Make documentation cohorts explicit execution inputs | `workspace_test_artifacts.py::doc_cohorts` | T |
| H021 | Next | Done; medium | Share Cargo selector construction for graph comparison | `workspace_feature_roots.py` | T |
| H022 | Next | Done; medium | Store measured scheduling weights as labelled data | `workspace_test_partitions.py::weight` | T |
| H023 | Next | Done; medium | Share cgroup ancestor and affinity inspection | `limited_verification.py` | T |
| H024 | First | Done; low | Construct optional Cargo job arguments explicitly | `scripts/dev.sh` | T |
| H025 | Next | Done; medium | Share physical filesystem/free-space probing | `check-scratch-space.py and workspace_test_storage.py` | T |
| H026 | Next | Done; medium | Share frozen execution and fixture validation | `test262_execution_batches.py::prepare/validate_resume` | T |
| H027 | Next | Done; medium | Separate snapshot verdict reconciliation stages | `test262_execution_batches.py::snapshot_verdicts` | T |
| H028 | Next | Done; medium | Organize module-boundary guards by subsystem | `scripts/check-module-boundaries.sh` | T |
| H029 | Next | Done; medium | Share CI bootstrap and cache setup | `.github/workflows regression jobs` | T |
| H030 | Later | Done; high | Avoid redundant full AOT CI prerequisites | `array and AOT regression workflows` | T |
| H031 | Later | Done; high | Give repeated CI regression selections one inventory | `ci.yaml and focused regression workflows` | T |
| H032 | Investigate | Done; high | Audit tooling-test modules against CI discovery | `tooling-contracts.yaml and scripts/tests` | I + T |
| H033 | Next | Done; low | Share chart display identity and count preparation | `generate-test-graph.py::render/render_svg` | T |
| H034 | Next | Done; medium | Make resource-number parsing contracts explicit | `limited_verification.py and check-scratch-space.py` | T |
| H035 | Later | Done; high | Separate build process discovery from affinity mutation | `workspace_build_cpus.py::expand_build_tree` | T |
| H036 | Next | Done; medium | Separate typed Test262 inputs, dispatch and rendering | `scriptw-cli/src/main.rs::handle_test262_command` | R |
| H037 | Next | Done; medium | Share option cursor/value extraction | `CLI Test262 and case-worker argument parsers` | R |
| H038 | Next | Done; medium | Declare allowed options per subcommand | `CLI cross-command option validation` | R |
| H039 | Later | Done; high | Replace positional shard ambiguity with an explicit option | `CLI parse_test262_args` | R |
| H040 | Next | Done; medium | Extract global options in one pass | `take_global_jobs/host_surface_policy/intl_profile` | R |
| H041 | Later | Done; high | Use one JSONC lexical pass | `strip_json_comments and strip_trailing_json_commas` | R |
| H042 | Next | Done; medium | Centralize complete and partial outcome totals | `CLI progress/status and publication counters` | R |
| H043 | Next | Done; medium | Define taxonomy wire labels and ordering once | `FailureKind, OutcomeKind, FailureOrigin` | R |
| H044 | Next | Done; medium | Share human report rendering | `Test262 and CLI count/summary printers` | R |
| H045 | Next | Queued; medium | Use structured engine failures before prose heuristics | `classify_engine_failure/classify_failure_outcome` | R |
| H046 | Next | Queued; medium | Type publication recovery conditions | `CLI publish-status missing/incomplete errors` | R |
| H047 | Next | Queued; medium | Validate README markers and replace output atomically | `rewrite_current_status_block` | R |
| H048 | Later | Queued; high | Publish JSON/TXT status as a coherent generation | `write_published_status_artifact` | R |
| H049 | Later | Queued; high | Encapsulate snapshot decoding and admission | `Test262 SnapshotFile and snapshot_publication` | R |
| H050 | Later | Queued; high | Reduce duplicate runner API entry points | `ConformanceRunner and free functions` | R |
| H051 | Later | Queued; high | Give each case worker one lifecycle owner | `ConformanceRunner execution/session/checkpoint closure` | R |
| H052 | Next | Queued; medium | Preserve primary case errors through worker cleanup | `child/reused-process timeout construction` | R |
| H053 | Later | Queued; high | Make materialized execution dispatch exhaustive | `execute_materialized_case` | R |
| H054 | Next | Queued; medium | Type stale-checkpoint envelope mismatch reasons | `ResumeCheckpointLoadError` | R |
| H055 | Investigate | Queued; high | Characterize and validate the pinned frontmatter subset | `parse_frontmatter_block/list/negative` | I + R |
| H056 | Investigate | Queued; high | Make module-catalog eligibility sound without extra lowering | `module_catalog.rs::ModuleLoadingSyntax` | I + R |
| H057 | Investigate | Queued; high | Distinguish module IO failures from absent imports | `module_catalog.rs::fixture_rows/resolve` | I + R |
| H058 | Next | Queued; medium | Bind module content kind and validated location once | `module_catalog.rs::Catalog::resolve` | R |
| H059 | Next | Queued; medium | Name module request key fields | `module_catalog.rs::Catalog::add_edge` | R |
| H060 | Next | Queued; medium | Share deterministic test/harness traversal | `scan_tests and scan_harness_files` | R |
| H061 | Next | Queued; medium | Move frontend tests into topic-owned child modules | `scriptw-front/src/lib.rs::tests` | R |
| H062 | Next | Queued; medium | Share UTF-8/UTF-16 Dynamic Function preparation | `scriptw-front/src/dynamic_function.rs` | R |
| H063 | Later | Queued; high | Make source coordinate units explicit | `frontend source_span and byte_offset helpers` | R |
| H064 | Later | Queued; high | Isolate UTF-16 sentinel decoding state | `utf16_source.rs::decode_js_string_utf16` | R |
| H065 | Next | Queued; medium | Use checked JSON cursor/digit primitives | `scriptw-front/src/json.rs` | R |
| H066 | Later | Queued; high | Separate program-lowering orchestration | `lower_script_program_with_allocations` | R |
| H067 | Investigate | Queued; high | Replace avoidable allocation-state clones with ownership transfer | `AnalysisAllocationState handoff in lowering` | I + R |
| H068 | Investigate | Queued; high | Audit and encode lowering source-length units | `ProgramIr source_len and lowering inputs` | I + R |
| H069 | Later | Queued; high | Organize IR domains and embedded tests | `scriptw-ir/src/ir.rs and analysis.rs` | R |
| H070 | Investigate | Queued; high | Share only identical generator protocol operations | `generator and async_generator lowering modules` | I + R |
| H071 | Investigate | Queued; high | Type identities where string aliases permit a real mix-up | `FunctionId and analysis owner/helper IDs` | I + R |
| H072 | Next | Queued; medium | Type the closed lowering invariant domain | `lowering.rs::new_program invariant strings` | R |
| H073 | Later | Queued; high | Separate oracle host and module responsibilities | `scriptw-spec-exec/src/lib.rs` | R |
| H074 | Later | Queued; high | Give oracle agent threads one lifecycle owner | `HostSession and AgentThreadState` | R |
| H075 | Next | Queued; medium | Scope oracle realm restoration | `HostRealmContext::with_context/eval` | R |
| H076 | Next | Queued; medium | Use the existing JSON serializer for oracle strings | `json_string_array/json_escape` | R |
| H077 | Investigate | Queued; high | Measure and index repeated embedded-module lookups | `embedded_module_graph.rs::find_module` | I + R |
| H078 | Later | Queued; high | Bind snapshot budget charges to operations | `rooted_snapshot capture and validation` | R |
| H079 | Next | Queued; medium | Centralize snapshot property/array ordering validation | `rooted_snapshot/validation.rs` | R |
| H080 | Investigate | Queued; high | Measure snapshot object identity lookup cost | `rooted_snapshot/capture.rs::Capture::object` | I + R |
| H081 | Next | Queued; medium | Reduce redundant Wasm emission wrapper chains | `aot-wasm emit.rs and runtime_artifact.rs` | R |
| H082 | Next | Queued; medium | Share program/runtime admission and linkage checks | `runtime_function/global_script/direct_eval emitters` | R |
| H083 | Next | Queued; medium | Type source-compilation import slots | `SourceCompilationProfileAdmission` | R |
| H084 | Next | Queued; medium | Check separate function/global index conversions | `RuntimeLayout::emit_runtime_imports` | R |
| H085 | Next | Queued; medium | Separate AOT tests by existing feature ownership | `aot-wasm root/planning/control_flow tests` | R |
| H086 | Next | Queued; medium | Split engine tests into domain-owned child modules | `scriptw-engine/src/tests.rs` | R |
| H087 | Next | Queued; medium | Share Script/Module setup using explicit parse goals | `Engine compile_script/module and run_script/module` | R |
| H088 | Next | Queued; medium | Share cache-limit parsing with explicit fallbacks | `cache.rs::parse_cache_limit/parse_tier_limit` | R |
| H089 | Investigate | Queued; high | Assess a focused runtime-image producer boundary | `runtime-image build.rs engine source includes` | I + R |
| H090 | Later | Queued; high | Bind admitted runtime and original budget at construction | `NativeSourceCompilerContext::new` | R |
| H091 | First | Done; medium | Use one LDML schema implementation including staged generation | `intl_ldml_schema.py and NumberFormat staging` | T + G |
| H092 | Next | Queued; medium | Retain parsed collator locale at admission | `CollationProfile::preferences` | G |
| H093 | Next | Queued; medium | Share locale lookup truncation | `collator profiles::matching and preferences::resolve` | G |
| H094 | Investigate | Queued; high | Specify allowed collator consumed-load inventory | `CollationProfile::construct` | I + G |
| H095 | Next | Queued; medium | Check collator marker inventory across interfaces | `collator_image MARKERS and projection Marker::ALL` | G |
| H096 | Next | Queued; medium | Name collator framing sizes, offsets and metadata bits | `collator protocol/image framing` | G |
| H097 | Next | Queued; medium | Make image export invariants explicit and consistent | `image_build list/segmenter/collator` | G |
| H098 | Next | Queued; medium | Validate generator owners and support isolated roots | `generate-intl-collator-data.py` | G |
| H099 | Later | Queued; high | Stage related generated outputs with recoverable publication | `collator and related identity generators` | G |
| H100 | Next | Queued; medium | Separate source/archive and feature/lock identity validation | `locale-collations and segmenter identity generators` | G |

## First-batch dependencies and acceptance

- **H001:** coordinate workspace/lockfile, engine API, CLI, Intl tests, identity
  guard and cohort anchor. Remove `ArtifactKind`, `Artifact.kind`, native-target
  compile options and obsolete emission routes; keep Wasmtime native execution.
  Advance only the program cache-key domain to v20. Prove the runtime-image
  auxiliary anchor has zero tests. Account for retired zero-test targets without
  deleting meaningful Wasm/oracle coverage. No dependency on other candidate hills.
- **H002–H004:** preserve former documents in same-directory dated history and
  retain incoming workflow anchors. Current inventory owns status; CONTINUE owns
  next actions; tasks links to permanent acceptance. Confirm the new GitHub URL
  before updating it. Keep the external website domain and frozen evidence intact.
- **H011:** keep options, command order, scheduling, isolation and receipt schemas
  intact. Pure parse/validation/inventory helpers precede a run-owned execution
  closure. Import must neither parse caller argv nor run commands/write files.
  Existing isolated policy tests retain their assertions after function relocation.
- **H017:** share defaults/validation only; retain per-caller root discovery and
  the driver's required manifest. Fake-publication controls must prove interruption,
  changed-source rejection, resume, original exit statuses and final publication.
- **H024:** keep Cargo selection/defaults, argument boundaries and exit statuses;
  use existing custom-target and spaced-path developer-wrapper controls.
- **H091:** canonical parser is `scripts/intl_ldml_schema.py`; NumberFormat copies
  it into its isolated staging directory and imports that name. Remove the duplicate.
  Bind changed generator owners in source identities. Require byte-identical
  semantic outputs; report legitimate provenance changes separately.

H001 and H091 must finish source/identity updates before costly Rust verification.
H011, H017 and H024 receive tooling checks before they supervise native work.
H002–H004 depend on the final actual batch state for completion claims.

## Verification record

All eight first-batch hills are complete in `ab7093a46`, `30065bbb5` and
`e7532de91`. The [verification record](rust-rewrite/maintenance-first-wave-20261021.md)
and [source-bound evidence](rust-rewrite/maintenance-first-wave-20261021.json)
retain exact inventories, artifact identities, failed attempts and final results.

Validation includes 653 tooling tests, locked all-target/all-feature Cargo check,
973 engine tests, 82 CLI unit tests, 856 CLI integration tests, the zero-test
runtime-image anchor, and 299 Intl integration tests from the preceding frozen
batch source. Full NumberFormat regeneration covers 1,082 locales with identical
semantic products. The fresh default compiler's original-case replays preserve
12 Success/4 Bug and ten passing nearby controls with the same four failures.

Exactly four obsolete C/native library/documentation scopes leave the inventory
(113 to 109); surviving command plans are identical. Original failed receipts
remain recorded. These checks do not claim a complete current-source workspace,
release or Test262 campaign. Canonical README totals remain at their complete
pre-repair baseline. At that checkpoint, the remaining 92 hills were candidates, not implemented work.

Raw local evidence is in `target/maintenance-first-wave-20261021`; the committed
record preserves its hashes and portable command paths.

## Tooling and documentation wave

H005–H010, H012–H016, H018–H023 and H025–H035 are complete in `73e87f962` (28 additional hills).
Their [change and verification record](rust-rewrite/maintenance-tooling-wave-20261021.md)
and [bounded evidence](rust-rewrite/maintenance-tooling-wave-20261021.json) bind
663 passing tooling controls, exact unchanged workspace plans, all 15 Intl
identities and maintained documentation/ownership checks. This wave depends on
the first batch and changes no product semantics or published Test262 totals.
H045–H090 and H092–H100 remain queued; all are authorized.

## CLI and reporting wave

H036–H044 are complete in `59dae70cf`. Their [verification record](rust-rewrite/maintenance-cli-wave-20261021.md)
and [bounded evidence](rust-rewrite/maintenance-cli-wave-20261021.json) preserve the
original full selected checkpoint and corrected 478 library, 89 CLI unit and
14 affected command passes. This wave depends on the tooling wave, changes the
shard command spelling, and preserves all taxonomy wire labels and published
Test262 totals. There are 45 completed hills and 55 authorized hills remaining.
