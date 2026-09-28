# Test262 failure backlog

The completed **2026-09-28** Wasm-AOT run recorded **99,229/102,956 passing executions** across **748/748 matrix sections**. All **3,727 failures** are assigned exactly once to the **117 tasks** below: 3,248 Bug, 442 NotImplemented and 37 Crash.

Root-cause assessments: **54 confirmed**, **50 suspected**, **13 unresolved**. A suspected or unresolved task is an investigation with source pointers, not a claim that its mechanism has been proven. Large groups can contain secondary failures exposed after the first cause is repaired.

232 executions are assigned to explicit dynamic-source AOT-boundary tasks. They remain failures in the denominator. Review finite-source specialization opportunities within those tasks; never turn this classification into a skip list.

## Working order

1. Claim one task by setting its registry status to `in-progress` and recording an assignee if useful.
2. Reproduce its exact modes, confirm the cause, and fix the general compiler or runtime operation.
3. Replay the whole task and adjacent families. Record a revision and native evidence in `resolution`, then mark `fixed`.
4. Keep all original IDs and diagnostics. Refresh conformance totals only with another complete published matrix.

Tasks with crashes come first, followed by required failure groups ordered by size, then dynamic-source boundary work. This is a triage order, not a dependency claim. Split a task when a minimal reproducer reveals independent mechanisms; move each execution to exactly one new owner and preserve existing task IDs.

| Task | Executions | Bug | NotImplemented | Crash | Cause | Status |
|---|---:|---:|---:|---:|---|---|
| [F001: Stop scratch/side-storage exhaustion in repeated allocation loops](F001-heap-retention-loops.md) | 18 | 0 | 0 | 18 | suspected | open |
| [F002: Throw TypeError instead of trapping on inherited RegExp methods](F002-regexp-primitive-receiver-trap.md) | 12 | 0 | 0 | 12 | unresolved | open |
| [F003: Investigate deep WeakMap construction and GC traversal timeout](F003-deep-weakmap-timeout.md) | 2 | 0 | 0 | 2 | unresolved | open |
| [F004: Profile the DateTimeFormat–Temporal calendar comparison timeout](F004-intl-dtf-timeout.md) | 2 | 0 | 0 | 2 | unresolved | open |
| [F005: Handle oversized RegExp replacement allocations through JavaScript completion](F005-oversized-string-allocation.md) | 2 | 0 | 0 | 2 | confirmed | open |
| [F006: Investigate large Unicode private-field class timeout](F006-unicode-private-class-timeout.md) | 1 | 0 | 0 | 1 | unresolved | open |
| [F007: Project non-ISO Temporal era and eraYear from the calendar kernel](F007-temporal-era-getters.md) | 492 | 492 | 0 | 0 | confirmed | open |
| [F008: Implement non-ISO CalendarDateUntil and relative rounding](F008-temporal-calendar-difference.md) | 324 | 324 | 0 | 0 | confirmed | open |
| [F009: Resolve M13 and leap-month codes by calendar instead of ISO month tables](F009-temporal-monthcode-input.md) | 318 | 318 | 0 | 0 | confirmed | open |
| [F010: Replace ISO-only Temporal calendar field projections](F010-temporal-calendar-getters.md) | 308 | 308 | 0 | 0 | confirmed | open |
| [F011: Resolve all accepted non-ISO eras in Temporal property bags](F011-temporal-era-input.md) | 286 | 286 | 0 | 0 | confirmed | open |
| [F012: Implement the missing Intl.DurationFormat family](F012-intl-durationformat.md) | 220 | 220 | 0 | 0 | confirmed | open |
| [F013: Implement the missing Intl.ListFormat family](F013-intl-listformat.md) | 160 | 160 | 0 | 0 | confirmed | open |
| [F014: Implement the missing Intl.Segmenter family](F014-intl-segmenter.md) | 154 | 154 | 0 | 0 | confirmed | open |
| [F015: Align RegExp Unicode property data with pinned Test262](F015-regexp-unicode-data.md) | 138 | 138 | 0 | 0 | suspected | open |
| [F016: Implement ShadowRealm intrinsic and callable boundary](F016-shadowrealm-intrinsics.md) | 124 | 124 | 0 | 0 | confirmed | open |
| [F017: Implement the missing Intl.DisplayNames family](F017-intl-displaynames.md) | 110 | 110 | 0 | 0 | confirmed | open |
| [F018: Convert and merge non-ISO calendar fields before storing ISO dates](F018-temporal-calendar-conversion.md) | 102 | 102 | 0 | 0 | confirmed | open |
| [F019: Represent generator yields in loops and statement control flow](F019-generator-control-flow-plan.md) | 67 | 0 | 67 | 0 | confirmed | open |
| [F020: Reject direct-eval var collisions in method parameter environments](F020-eval-parameter-declarations.md) | 48 | 48 | 0 | 0 | suspected | open |
| [F021: Enforce uninitialized lexical state on reads and writes](F021-lexical-initialization-state.md) | 46 | 46 | 0 | 0 | suspected | open |
| [F022: Use calendar-aware addition and subtraction for every Temporal carrier](F022-temporal-calendar-add.md) | 44 | 44 | 0 | 0 | confirmed | open |
| [F023: Compile recognized Unicode property patterns instead of unsupported execution](F023-regexp-unicode-property-admission.md) | 36 | 36 | 0 | 0 | suspected | open |
| [F024: Select calendar-correct MonthDay reference dates](F024-temporal-monthday-reference.md) | 34 | 34 | 0 | 0 | confirmed | open |
| [F025: Order async-generator destructuring and iterator suspension points](F025-async-generator-destructuring-plan.md) | 26 | 0 | 26 | 0 | confirmed | open |
| [F026: Lower suspension in for/while loop heads and abrupt control](F026-async-loop-head-await.md) | 25 | 0 | 25 | 0 | confirmed | open |
| [F027: Implement finite-input transcendental Math operations](F027-math-finite-functions.md) | 24 | 24 | 0 | 0 | confirmed | open |
| [F028: Compile finite Function constructor calls through subclass super](F028-dynamic-subclass-specialization.md) | 16 | 0 | 16 | 0 | suspected | open |
| [F029: Capture class names, private names and lexical super/newTarget in eval](F029-class-eval-context.md) | 14 | 14 | 0 | 0 | suspected | open |
| [F030: Preserve UTF-16 code units and surrogate pairs in RegExp matching](F030-regexp-utf16-matching.md) | 14 | 14 | 0 | 0 | suspected | open |
| [F031: Enforce calendar-specific MonthDay required fields before range checks](F031-temporal-monthday-required-fields.md) | 14 | 14 | 0 | 0 | confirmed | open |
| [F032: Lower await nested in export declarations and catch flow](F032-top-level-await-export.md) | 13 | 0 | 13 | 0 | confirmed | open |
| [F033: Add resumable for-in enumeration](F033-async-for-in-await.md) | 12 | 0 | 12 | 0 | confirmed | open |
| [F034: Preserve iterator records across awaited for-of heads](F034-async-for-of-head-await.md) | 12 | 0 | 12 | 0 | confirmed | open |
| [F035: Allow nested await within for-await-of bodies](F035-for-await-body-await.md) | 12 | 0 | 12 | 0 | confirmed | open |
| [F036: Compile Unicode properties of strings and RGI emoji](F036-regexp-string-properties.md) | 12 | 12 | 0 | 0 | confirmed | open |
| [F037: Enforce Unicode-mode RegExp early syntax errors for dynamic patterns](F037-regexp-unicode-syntax.md) | 12 | 12 | 0 | 0 | suspected | open |
| [F038: Implement Turkish, Azeri, and Lithuanian locale casing](F038-string-localecase-mapping.md) | 12 | 12 | 0 | 0 | confirmed | open |
| [F039: Key template objects by parsed site and realm across eval](F039-template-site-realm-cache.md) | 12 | 12 | 0 | 0 | suspected | open |
| [F040: Specialize literal cross-realm eval and empty Function constructors](F040-cross-realm-finite-source.md) | 8 | 0 | 8 | 0 | confirmed | open |
| [F041: Keep method lexical environments distinct for eval declaration checks](F041-eval-body-lexical-conflicts.md) | 8 | 8 | 0 | 0 | suspected | open |
| [F042: Retain global class and lexical bindings across evalScript](F042-global-lexical-state.md) | 7 | 7 | 0 | 0 | suspected | open |
| [F043: Implement source-phase module bindings and source objects](F043-source-phase-module-objects.md) | 7 | 7 | 0 | 0 | confirmed | open |
| [F044: Dispose await-using resources at each loop scope exit](F044-await-using-loop-disposal.md) | 6 | 6 | 0 | 0 | suspected | open |
| [F045: Infer function names from computed and numeric property keys](F045-computed-function-names.md) | 6 | 6 | 0 | 0 | suspected | open |
| [F046: Honor RegExp prototype getter special cases and generic toString](F046-regexp-prototype-receivers.md) | 6 | 6 | 0 | 0 | suspected | open |
| [F047: Route String.localeCompare through Intl.Collator](F047-string-localecompare.md) | 6 | 6 | 0 | 0 | confirmed | open |
| [F048: Accept callable Proxy objects in Function.prototype.bind](F048-bind-callable-proxy.md) | 4 | 4 | 0 | 0 | confirmed | fixed |
| [F049: Initialize class-name environments at the correct phase](F049-class-name-initialization.md) | 4 | 4 | 0 | 0 | suspected | open |
| [F050: Support super and parenthesized property destructuring targets](F050-destructuring-property-target.md) | 4 | 0 | 4 | 0 | confirmed | open |
| [F051: Emit modules whose only statements are import/export declarations](F051-empty-module-ir.md) | 4 | 0 | 4 | 0 | suspected | open |
| [F052: Refresh decimal numbering data to include Tolong Siki (tols)](F052-intl-numbering-data.md) | 4 | 4 | 0 | 0 | confirmed | open |
| [F053: Observe inherited and mutated replacer array elements](F053-json-replacer-array-get.md) | 4 | 4 | 0 | 0 | suspected | open |
| [F054: Use ECMAScript ToInt32 for large parseInt radix arguments](F054-parseint-radix-int32.md) | 4 | 4 | 0 | 0 | confirmed | fixed |
| [F055: Parse nested property and super destructuring assignment targets](F055-parser-destructuring-targets.md) | 4 | 4 | 0 | 0 | suspected | open |
| [F056: Allocate builtin-created arrays in the active function realm](F056-realm-created-array-prototypes.md) | 4 | 4 | 0 | 0 | suspected | open |
| [F057: Escape RegExp source for slash and line terminators](F057-regexp-source-escaping.md) | 4 | 4 | 0 | 0 | confirmed | fixed |
| [F058: Match the full ECMAScript whitespace set in all RegExp paths](F058-regexp-unicode-whitespace.md) | 4 | 4 | 0 | 0 | suspected | open |
| [F059: Lower spread arguments through RegExp lastIndex helper calls](F059-spread-call-lowering.md) | 4 | 0 | 4 | 0 | confirmed | open |
| [F060: Validate every locale passed to locale case conversion](F060-string-localecase-validation.md) | 4 | 4 | 0 | 0 | confirmed | open |
| [F061: Evaluate assignment RHS before null-super PutValue failure](F061-super-reference-order.md) | 4 | 4 | 0 | 0 | confirmed | open |
| [F062: Preserve Symbol results returned by ToPrimitive during ToPropertyKey](F062-symbol-property-key-coercion.md) | 4 | 4 | 0 | 0 | suspected | open |
| [F063: Keep non-ISO reference date fields when calendarName is never](F063-temporal-reference-string.md) | 4 | 4 | 0 | 0 | confirmed | fixed |
| [F064: Track Await grammar parameters correctly inside class field initializers](F064-class-field-await-grammar.md) | 3 | 3 | 0 | 0 | suspected | open |
| [F065: Carry class strictness through constructors, heritage and nested functions](F065-class-strict-context.md) | 3 | 3 | 0 | 0 | suspected | open |
| [F066: Parse let and escaped contextual identifiers as labels where allowed](F066-let-label-grammar.md) | 3 | 3 | 0 | 0 | suspected | open |
| [F067: Protect restricted global names during declarations and assignment](F067-restricted-global-properties.md) | 3 | 3 | 0 | 0 | suspected | open |
| [F068: Respect reference strictness when a super property write fails](F068-super-sloppy-failed-set.md) | 3 | 3 | 0 | 0 | suspected | open |
| [F069: Copy Annex B function declarations into the actual arguments binding](F069-annexb-arguments-copy.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F070: Use throwing Set for Array.prototype.fill writes](F070-array-fill-set-throw.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F071: Close Array.from iterators on mapping or property-creation errors](F071-array-from-iterator-close.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F072: Accept valid subclass ArrayBuffers returned by proxied species constructors](F072-arraybuffer-species-result.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F073: Preserve AsyncFromSyncIterator promise resolution and job order](F073-async-from-sync-promise-ticks.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F074: Preserve undefined argument conversion in boxed String concat](F074-boxed-string-concat.md) | 2 | 2 | 0 | 0 | unresolved | open |
| [F075: Honor Date default-hint primitive conversion after method overrides](F075-date-default-primitive.md) | 2 | 2 | 0 | 0 | unresolved | open |
| [F076: Implement the pinned staging non-ISO Date parsing expectations](F076-date-staging-noniso.md) | 2 | 2 | 0 | 0 | confirmed | open |
| [F077: Defer Object.getOwnPropertyDescriptor argument errors to runtime](F077-descriptor-primitive-coercion.md) | 2 | 0 | 2 | 0 | confirmed | open |
| [F078: Resolve JSON imports with attributes obtained through Proxy enumeration](F078-dynamic-import-proxy-attributes.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F079: Discover module imports inside compiled literal eval](F079-eval-import-graph.md) | 2 | 0 | 2 | 0 | confirmed | open |
| [F080: Publish RegExp legacy match state from compiled eval execution](F080-eval-regexp-legacy-state.md) | 2 | 2 | 0 | 0 | unresolved | open |
| [F081: Capture initialized for-let head bindings before per-iteration cloning](F081-for-head-capture-initialization.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F082: Resolve arguments to the correct function activation binding](F082-function-arguments-environment.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F083: Lower yield in generator method defaults and nested expressions](F083-generator-method-yield.md) | 2 | 0 | 2 | 0 | confirmed | open |
| [F084: Forward Iterator.from wrapper next results without extra validation](F084-iterator-from-forward-result.md) | 2 | 2 | 0 | 0 | confirmed | open |
| [F085: Split oversized emitted Wasm functions](F085-large-wasm-function.md) | 2 | 2 | 0 | 0 | confirmed | open |
| [F086: Keep frozen function caller/arguments values stable](F086-legacy-function-descriptor-invariants.md) | 2 | 2 | 0 | 0 | confirmed | open |
| [F087: Use one Math property installation order across realms](F087-math-realm-property-order.md) | 2 | 2 | 0 | 0 | confirmed | open |
| [F088: Preserve module dependency parse errors as resolution failures](F088-module-negative-phase.md) | 2 | 2 | 0 | 0 | confirmed | open |
| [F089: Preserve parenthesized assignment syntax for function naming](F089-parenthesized-namedevaluation.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F090: Construct Array copy-method errors in the builtin defining realm](F090-realm-array-copy-errors.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F091: Install and execute change-by-copy Array methods in created realms](F091-realm-array-copy-methods.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F092: Use IsRegExp and constructor identity before reading internal slots](F092-regexp-constructor-isregexp.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F093: Compile runtime-built Unicode ignoreCase matcher variants](F093-regexp-dynamic-ignorecase.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F094: Compile valid escaped Unicode-mode RegExp patterns](F094-regexp-escaped-grammar.md) | 2 | 2 | 0 | 0 | unresolved | open |
| [F095: Use RegExpBuiltinExec when an actual RegExp has noncallable exec](F095-regexp-exec-fallback.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F096: Represent RegExp quantifier bounds beyond machine-sized eager expansion](F096-regexp-large-quantifier.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F097: Use syntactically valid native function display names](F097-regexp-native-string.md) | 2 | 2 | 0 | 0 | confirmed | open |
| [F098: Read RegExp replacement match properties in spec order](F098-regexp-replace-get-order.md) | 2 | 2 | 0 | 0 | confirmed | open |
| [F099: Correct RegExp split species and lastIndex protocol](F099-regexp-split-trace.md) | 2 | 2 | 0 | 0 | unresolved | open |
| [F100: Preserve valid UTF-16/WTF-8 through replaceAll result construction](F100-replaceall-string-encoding.md) | 2 | 2 | 0 | 0 | unresolved | open |
| [F101: Exclude Symbol keys before proxy descriptor traps in object rest](F101-rest-excluded-symbol.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F102: Throw when GetFunctionRealm reaches a revoked Proxy newTarget](F102-revoked-newtarget-realm.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F103: Select SharedArrayBuffer fallback prototype from newTarget realm](F103-shared-buffer-realm-prototype.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F104: Implement Temporal.Duration locale formatting through DurationFormat](F104-temporal-duration-localestring.md) | 2 | 2 | 0 | 0 | confirmed | open |
| [F105: Check MonthDay supplied-year limits for every accepted calendar](F105-temporal-monthday-year-range.md) | 2 | 2 | 0 | 0 | confirmed | open |
| [F106: Resolve the Pacific/Johnston primary-identity disagreement](F106-temporal-timezone-link.md) | 2 | 2 | 0 | 0 | suspected | open |
| [F107: Resolve Annex B block function storage consistently](F107-annexb-binding-plan.md) | 1 | 0 | 1 | 0 | confirmed | open |
| [F108: Return observable iterator-result properties for arguments iteration](F108-arguments-iterator-result.md) | 1 | 1 | 0 | 0 | unresolved | open |
| [F109: Evaluate destructuring assignment references before source property reads](F109-destructuring-reference-order.md) | 1 | 1 | 0 | 0 | suspected | open |
| [F110: Preserve undefined values on optional-chain short circuits](F110-optional-chain-undefined.md) | 1 | 1 | 0 | 0 | unresolved | open |
| [F111: Respect Proxy receiver descriptors before defining a property](F111-proxy-receiver-descriptors.md) | 1 | 1 | 0 | 0 | suspected | open |
| [F112: Avoid unintended Proxy trap accesses during primitive conversion](F112-proxy-toprimitive-lookups.md) | 1 | 1 | 0 | 0 | unresolved | open |
| [F113: Remove with object environment after leaving its statement](F113-with-scope-exit.md) | 1 | 1 | 0 | 0 | suspected | open |
| [F114: Preserve boxed String length attributes through with assignment](F114-with-string-exotic-length.md) | 1 | 1 | 0 | 0 | suspected | open |
| [F115: Account for runtime eval source without a compiled specialization](F115-aot-runtime-eval.md) | 151 | 0 | 151 | 0 | confirmed | open |
| [F116: Account for runtime Function source without a compiled specialization](F116-aot-runtime-function.md) | 80 | 0 | 80 | 0 | confirmed | open |
| [F117: Account for runtime AsyncGeneratorFunction source without a compiled specialization](F117-aot-runtime-asyncgeneratorfunction.md) | 1 | 0 | 1 | 0 | confirmed | open |

## Repository check follow-up

[Repository check findings](evidence/repository-checks.json) record nine inherited Rust assertion failures and the existing module-size, formatting and identity-check failures found during this cleanup. They have Rxxx IDs, owners, source evidence and next steps. These are separate from the Test262 baseline above; the failing assertions remain enabled.

## Evidence and maintenance

[registry.json](registry.json) is the editable task registry. The Markdown and per-task execution lists are generated from it. Edit the registry, then render and validate:

```sh
python3 scripts/check-failure-backlog.py --write
./scripts/check-task-plan.sh
```

The check rejects missing, duplicated or invented executions, changed frozen evidence, mismatched outcome totals, and drifted task documents or replay lists. Marking a task fixed does not delete its original failure membership.

- [Baseline and hashes](evidence/baseline.json): pins, compiler and source identity, exact totals.
- [Original diagnostics](evidence/failures.json): every outcome, path, mode, detail and duration.
- [Extraction provenance](evidence/provenance.json): hashes and counts from all native matrix leaves, with zero exclusions.
- [Aggregate](evidence/aggregate.json) and [publisher output](evidence/published-status.json): frozen full-run evidence.
- [Publication session](evidence/publication-session.json): observed inputs and verified matrix completion.

The compiler was built from a dirty worktree. Its observed source-input and binary hashes identify the tested build; the recorded Git commit alone does not reproduce it. Source line references describe the triage checkout and may move as repairs land. This backlog stores the complete failed-execution evidence; the native passed-case inventories remain in the source snapshot directory recorded by the baseline. Per-leaf hashes are retained for auditing.

The old T00–T29 plans have been removed. Their IDs remain domain taxonomy in [conformance ownership](../docs/rust-rewrite/conformance-ownership.md) for compiler-owned reports and the CLI ledger; F001 and later IDs identify the actionable tasks here.

## Replay limits

Each task gives an exact-list replay command using two case workers and one compiler job per case. Run expensive verification within the current limit of 10 CPUs and half the machine's physical RAM, including all child processes. For Linux with a user systemd manager:

```sh
# Set the aggregate limit from this machine's physical memory.
test_ram_limit=$(awk '/^MemTotal:/ {printf "%.0f", $2 * 1024 / 2}' /proc/meminfo)
systemd-run --user --scope -p AllowedCPUs=0-9 -p CPUQuota=1000% \
  -p "MemoryMax=$test_ram_limit" -p MemorySwapMax=0 \
  python3 scripts/replay-test262-executions.py tasks/cases/F001.executions \
    --binary target/release/lila --suite-root test262/vendor/test262 \
    --output-dir target/test262-scratch/F001-replay --workers 2
```

Choose up to ten available CPUs if this host has a different affinity. The replay script freezes the compiler and execution list. Use `--resume` only for the same binary and inputs; use a fresh directory after a fix. No full suite was rerun while creating this backlog.
