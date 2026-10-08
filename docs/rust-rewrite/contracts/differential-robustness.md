# Bounded product robustness attempts

Status: source, controls and integration authored; no builds, tests, formatting or
campaigns executed. This does not establish a sustained zero-panic campaign or
T25 acceptance.

`RobustnessInput` owns a closed target, positive attempt deadline and exact native
bytes. Its strict version-one wire uses lowercase hex, so invalid UTF-8 and NUL
remain reproducible rather than being replaced or escaped into a different input.
Native decoding and mutation remain bounded by the existing worker request limit.
The byte allowance is one quarter of that limit, leaving room for hex and envelope
encoding; campaign count uses the existing 1–128 bound and rejects seed overflow.
Every no-payload target uses an empty struct wire variant, so a foreign `goal`,
`parser` or arbitrary target field rejects instead of disappearing during serde
admission. Valid version-one target bytes and fingerprints remain unchanged.

The SDK and CLI use the original selected differential worker image, source/image
provenance checks, deadline before spawn, private staging, bounded journal and
group/direct-child retirement. One extracted transport method owns both request
domains. Robustness requests have separate checked bindings and header/stage/terminal
frames; a differential frame cannot complete a robustness attempt or vice versa.
Bounded stderr is retained too, including panic diagnostics. Worker request parsing
occurs after spawning; arbitrary JavaScript/native target inputs are never run in
the controller. Future execution must use the existing one-CPU, 4096-MiB, zero-swap
outer launcher; the worker inherits that resource boundary with no uncapped fallback.

Compiler Script/Module targets call `Engine::inspect_compilation`, which consumes
the actual uncached preparation, original IR lowering, Wasm emitter and selected
product Wasmtime validator. It has no interpreter, alternate backend or cache-hit
shortcut. Before each actual stage the worker commits a frame. A timeout, crash or
killed worker therefore retains the last entered stage, not an inferred completed
stage. Runtime setup and invalid emitted Wasm remain separate outcomes. Parse,
early-error and resolution diagnostics keep their actual typed phases. The existing
frontend's caught-parser-abort capability error stays unsupported and red.

Independent native targets consume the original Test262 frontmatter/execution-plan
parser, snapshot schema/provenance/checkpoint decoder, corpus decoder, exact v4
embedded resolver/catalog constructor, or schema-v3 native worker frame/terminal
admission. The snapshot byte decoder is shared with the original file reader.
Frontmatter parsing does not execute includes or syntax. Embedded resolver inputs
cannot access ambient files. The report target takes the original serialized
`WorkerFrame::Terminal` and checks its execution payload against the original
schema-v3 terminal domain. It cannot construct a completed backend observation or
admit provenance, transcript counts or arbitrary full comparison reports.

The independent `ir-admission` target calls `Engine::inspect_ir_compilation`.
Its native version-one JSON envelope contains only `schema_version` and `body`.
The input boundary limits the original bytes to 16,384 before JSON decoding,
keeps the decoder's default recursion limit, then checks at most 256 JSON values,
16 nested JSON levels and 512 UTF-8 bytes per string or key before retaining its
private native tree. Unknown envelope fields and schemas reject before candidate
IR allocation. Each operation and operand accepts exactly its named fields;
unknown operations, extra fields and unsupported kinds reject during admission.

The bounded body admits these self-contained statement forms:

- `empty`, or `value` with a literal operand;
- `define_property`, with a literal `target`, static string `key` and literal
  `value`, through the original `ObjectPropertyDefinitionIr::new`;
- `delete_optional`, with a literal `target`, Boolean `strict` and ordered `chain`,
  through the original `DeleteOptionalPropertyChainIr::new`;
- `block` with a `body`, or `if` with a literal `condition` and `then`/`else` bodies;
- `await` and `yield` with a literal `value`, as rejection probes consumed by the
  original `SynchronousLoopBodyIr::new` over the complete candidate body.

Literal `kind` is `undefined`, `null`, `object` (an empty ordinary object),
`boolean` or `string` with `value`, or `number` with exactly 16 lowercase hex
`bits`. Numeric bit patterns, including signed zero and NaN payloads, are retained.
The decoder derives operand kind and possible-kind information; the wire cannot
supply those facts independently. Optional-chain `property` operations have a
static string `key` and Boolean `shorted`. `call` operations have literal `args`,
Boolean `shorted` and `boundary_before`, and the existing Reference-or-undefined
receiver contract. An empty or call-terminal delete chain is a real checked
constructor rejection, as is a property definition on a non-object target.

The Engine first records `IrInput`, then prepares and lowers a genuinely empty
Script through its original compiler at `Preparation` and `Lowering`.
`IrAdmission` constructs the checked operations and complete synchronous body
before installing anything. Rejection leaves the lowered Program unchanged,
including when suspension appears in an unselected branch. Only that empty
Script's body can change: original source, environment, template, global and
function ownership remain intact. The wire has no function IDs, source IDs,
private brands, captures, scope allocation or continuation certificates. Await
and Yield cannot pass synchronous admission and never reach emission.

Accepted candidates use the same `Emission`, `RuntimeSetup` and `Validation`
implementation as source compilation. No candidate executes. Input/admission
errors retain `NativeBoundary`; only a validated emitted Wasm artifact completes
acceptance. This covers the three checked constructors above and ordinary emission
of their bounded surrounding blocks and branches. It does not cover arbitrary
ProgramIr, source-owned resumable domains or an independent optimizer. Existing
Script/Module robustness targets continue to exercise real lowering, including
its folding and flow analysis; this native target does not replace that path.

For example, native bytes in `repro.ir.json` may be:

```json
{"schema_version":1,"body":[{"op":"define_property","target":{"kind":"object"},"key":"answer","value":{"kind":"number","bits":"4045000000000000"}}]}
```

Run them through the existing campaign command using `--input repro.ir.json
--target ir-admission`. The ordinary exact-byte mutation, replay and crash reducer
retain the original native bytes, target and deadline, including mutations that
break JSON or the closed IR domain. They do not reserialize, repair or reinterpret
mutated input as JavaScript. The IR target and its controls are authored source;
compilation and focused validation remain pending for the joined checkpoint.

JSON, RegExp, Number, BigInt and Temporal-date targets turn data into one native JSON
String literal inside a fixed product Script. They compile and validate through the
same pipeline, then run those exact emitted bytes in the original fresh Wasmtime
Store. They contain no eval, user source splicing, external IO or privileged host.
Original normal/throw completion and its safe structured representation are retained;
rejecting data is not forced to succeed. A missing builtin/compiler/runtime capability
is unsupported or failed evidence, never an invented accepted observation. URI
calls use the additive bounded UTF-16 input described below. Earlier URI stress
reproducers are unchanged and are not used by these targets.

The `encode-uri`, `encode-uri-component`, `decode-uri` and
`decode-uri-component` targets accept a strict native JSON payload:

```json
{"schema_version":1,"units":[37,50,70]}
```

Only these two fields are admitted. The payload is at most 64 KiB and contains
at most 4,096 unsigned UTF-16 units. Every unit becomes an explicit `\uXXXX`
escape in one fixed String literal; NUL, quotes, backslashes and lone surrogates
remain data. The selected original URI builtin runs through real compilation,
validation and execution. Normal and throw completions retain their original
safe representation. The target does not normalize invalid percent sequences,
replace surrogates or insert user source. `BuiltinInput` precedes compilation,
and malformed native payloads reject there without claiming builtin execution.

The `prelude` target uses the original `parse_test_executions`, `load_preludes`
and `materialize_test` authorities. Its native version-one payload contains
`source`, `execution_mode`, `harness_profile`, `merged_harness`, `files` and
optional `overrides`. A file has only `name` and `contents`. The exact selected
mode must be one of the original frontmatter's admitted modes. Profiles are
`none`, `custom_merged` and `embedded_wasm_aot`; only `custom_merged` admits a
bounded non-null `merged_harness`. The input cannot supply host ownership,
completed preludes or rewritten source.

```json
{"schema_version":1,"source":"/*---\nincludes: [helper.js]\n---*/\n1;","execution_mode":"strict-script","harness_profile":"none","merged_harness":null,"files":[{"name":"assert.js","contents":"/* assertion fixture */"},{"name":"helper.js","contents":"/* declared include */"}],"overrides":[]}
```

Source is at most 16 KiB, merged harness at most 16 KiB, and the combined file
and override lists contain at most 16 entries and 64 KiB of contents. The raw
payload is at most 192 KiB. Fixture names are normal relative paths of at most
256 bytes, with no empty, dot, parent, device, backslash or NUL components.
Files are created only below a private worker-owned suite. The original loader
decides named-section precedence and required preludes. Overrides use the
original `PreludeStore::insert`, which invalidates embedded host ownership when
`assert.js` or `sta.js` is replaced. Thus an embedded profile may materialize its
genuine complete host; candidate bytes cannot manufacture that authority.
Original strict/raw/Module placement, include deduplication, missing includes
and host requirements remain observable. Materialized output is limited to
512 KiB across case, Module and agent preludes. Acceptance means materialization
completed; it does not execute the case or establish Test262 conformance.

The `filesystem-resolver` target consumes the real
`FilesystemModuleLoader` and checked `ModuleRequestKeyIr`. Its native payload is:

```json
{"schema_version":1,"operation":"resolve_and_load","referrer":"entry","layout":"plain","specifier":"./dep.js","attributes":[]}
```

The closed operations are `resolve_and_load` and `load_direct`; referrers are
`none`, `entry` and `nested`, and layouts are `plain` and `symlinks`. Direct load
requires no referrer or attributes. Resolution attributes contain only `key`
and `value` and pass the original canonical constructor, including duplicate
rejection. The raw payload is at most 4 KiB, specifier at most 1 KiB and attributes
at most eight. It cannot choose fixture contents or an ambient root.

Each invocation creates six small fixed files inside a private sandbox. An
owned sibling directory provides a real escape-denial candidate. On Unix the
symlink layout adds an in-root alias and a link to that owned sibling; inability
to create these links is explicit unavailability. Other platforms refuse the
symlink layout. Resolve/load invokes the original confinement checks, and a
successful returned key must remain under the canonical root with unchanged
identity. Direct load deliberately permits a candidate outside key to reach
the original load-time denial. No rejected path is read by the probe itself,
and no Module body executes. JSON attribute support follows the original loader,
rather than being independently inferred by this target.

Prelude stages are `Decode`, `PreludeInput`, `PreludeLoad` and
`PreludeMaterialization`. Filesystem stages are `Decode`, `FilesystemInput`,
`FilesystemSetup`, `ModuleResolution` and `ModuleLoading`. Original boundary
rejections remain `NativeBoundary`; fixture IO failures, unavailable symlink
support and cleanup failures remain separate red results. RAII removes owned
fixtures on error, explicit successful cleanup is checked, and the existing
controller removes the complete staging tree after worker exit or termination.
Neither target can publish artifact bytes or builtin execution evidence.

The deterministic mutation algebra deletes, truncates, replaces, inserts or
duplicates bounded byte ranges. First attempt retains the original input. Every
mutation retains its descriptor and exact new bytes; later attempts keep target
and deadline. These mutations intentionally include malformed syntax, metadata and
encodings. The result records accepted, rejected, unsupported, failed, invalid-Wasm
and worker-failure domains separately. `completed_without_failure` means that this
bounded boundary returned normally, including a structured rejection. It does not
claim source acceptance, semantic agreement, fuzz completeness or conformance.

Fresh campaign output writes `base.bin`, the base native input, each exact `.bin`
and replay input before executing it, then the actual observation with raw committed
journal/diagnostic bytes. The synchronized aggregate begins running with all seeds
pending. Cancellation, worker failure and capability failures cannot publish finished;
the first failed attempt stops remaining cases. The original durable write/atomic
checkpoint helpers are shared with semantic campaigns. Every failed/rejected input
is retained; no input is promoted into the semantic executable corpus.

```sh
lila differential robustness --input repro.js --target script --seed 1 --cases 32 \
  --timeout-ms 300000 --output-dir fresh-robustness --oracle spec-exec
lila differential replay-robustness fresh-robustness/case-000.input.json --oracle spec-exec
lila differential minimize-robustness fresh-robustness/case-000.input.json \
  --replays 128 --output-dir fresh-reduction --oracle spec-exec
lila differential replay-robustness fresh-reduction/minimized.input.json --oracle spec-exec
```

`scripts/run-robustness-campaign-tier.py` now consumes the existing CLI through
one confirmed verification scope. Its PR tier selects seven native decoder,
prelude and filesystem targets with two cases each; nightly selects all nineteen
targets with sixty-four cases each. Both tiers retain identical original seed
bytes and nonoverlapping seed ranges for their shared targets. The snapshot seed
is explicitly a legacy decoder fixture, not current conformance evidence.
The existing 300,000 ms robustness-attempt budget is fixed in each request;
the separate generated differential grammars keep their original deadlines.

The driver writes every original input before starting any command, and atomically
records each pending/running/completed/failed target in `tier.json`. Each CLI runs
serially and retains its own original native observations, exact mutated bytes,
aggregate and logs. Independent targets continue after a failure. A nonzero exit,
missing case evidence, wrong original input/deadline, foreign image or incomplete
Rust aggregate keeps the tier red. Existing output directories are refused.
Interruption retains pending work; it cannot publish a completed tier. The driver
does not interpret a normal parser rejection as source acceptance or conformance.

```sh
python3 scripts/limited_verification.py --memory-mib 4096 -- \
  python3 scripts/run-robustness-campaign-tier.py ./target/debug/lila fresh-tier pr-fast
```

The differential CI workflow connects these controls and both debug/optimized
tiers to the existing feature-enabled builds and artifact retention. Source,
driver controls and wire controls are authored; execution and sustained campaign
acceptance remain deferred to the joined verification checkpoint.

After the combined type checkpoint, the focused new controls run serially:

```sh
python3 scripts/limited_verification.py --memory-mib 4096 -- python3 -m unittest discover -s scripts/tests -p 'test_limited_verification.py'
python3 scripts/limited_verification.py --memory-mib 4096 -- python3 -m unittest discover -s scripts/tests -p 'test_robustness_campaign_tier.py'
python3 scripts/limited_verification.py --memory-mib 4096 -- cargo test --locked -p lila-test262 --features spec-exec-oracle --lib differential::robustness::tests::every_native_target_rejects_foreign_fields_without_changing_its_existing_wire -- --exact
```

The driver fixtures test original-byte retention, matching tier seeds, failed or
foreign aggregate refusal, independent target continuation, interruption and no
resume. They mock process results and establish no product or campaign result.
The resource fixtures test cache clamping, retention of stricter caller values
and pre-execution refusal of malformed values. The Rust control exercises the
actual native request decoder for all eight empty-payload targets.

The separate crash reducer replays the baseline once, then consumes the existing
checked candidate replay limit (1–512). Only a bound, fully committed stage prefix
without a terminal may establish a timeout or abnormal-exit witness. The worker
must have admitted the exact request and entered the target; missing headers,
foreign provenance, torn frames, completed terminals, cleanup failures, truncated
IO evidence and supervisor/admission failures cannot establish this witness.
Ordinary structured parser/native rejection is explicitly not reducible. Existing
semantic reducers continue to stop on every worker failure.

Candidates delete contiguous byte chunks or lower an existing byte to zero or its
integer half. Each must strictly decrease the lexicographic pair of byte length
and byte sum, with exact raw bytes retained even when syntax/UTF-8 becomes invalid.
Target, Script/Module goal, deadline and fixed target policy stay identical. A
candidate is retained only with the same actual compiler/worker identity, last
entered stage and exact timeout or exit-code/signal disposition. A different
valid result is rejected as a reduction candidate. Incomplete/foreign evidence,
controller errors and unavailable target outcomes stop further attempts. Every
candidate's native request and `.bin` precede the selected-worker call, and its
actual journal/stderr observation precedes the keep/reject decision. Callback
counts include only invoked attempts; the baseline is separate from the finite
candidate budget. Cancellation and budget exhaustion preserve the best actual
witness already observed.

`reduction.json` retains all decisions, including pending/interrupted work, and
the exact baseline signature. `minimized.input.json` and `minimized.bin` are
written only after a real crash/timeout baseline has been observed. The native
input replays with the existing command above, including its original deadline;
it remains failed evidence outside the executable semantic corpus. Exhausting
this deterministic candidate set is not a global minimum claim. The minimizer
prints the report and exits nonzero because the retained failure is unresolved;
it has no corpus-green or conformance verdict. Controls for all these source
boundaries are authored and unrun.

Target names are `script`, `module`, `ir-admission`, `frontmatter`, `snapshot`, `corpus`,
`module-graph`, `report-observation`, `json`, `regexp`, `number`, `bigint` and
`temporal-date`, plus `encode-uri`, `encode-uri-component`, `decode-uri`,
`decode-uri-component`, `prelude` and `filesystem-resolver`. The explicit oracle
flag selects the existing developer worker
feature/build; robustness targets execute only the product/native boundary and do
not run the oracle interpreter. Feature-off/platform refusal remains explicit.
Private controls cover native wire damage, deterministic mutation bounds, phase and
terminal proof, exact evidence, cancellation and worker failure. Actual selected-worker
controls cover compilation, parser rejection, independent native targets, builtin
throws and crash diagnostic retention. The independent IR slice now passes five
IR controls, 19 robustness library controls, two compiler-inspection controls
and two actual worker/CLI native-input replay controls. Its empty struct wire
variant rejects foreign fields while preserving the `ir_admission` kind. These
focused results do not establish the complete robustness cohort or sustained
campaign acceptance.

The URI/prelude/filesystem successor includes native payload, mode, original
host-ownership, fixture confinement, symlink, cleanup and terminal-domain controls,
plus actual selected-worker and CLI replay controls. They are authored and unrun
pending the joined source checkpoint. They retain the existing exact-byte mutation,
reducer, crash-prefix and durable report algorithms; no sustained campaign or new
green milestone is claimed.
