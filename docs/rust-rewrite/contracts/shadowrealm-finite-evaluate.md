# Finite ShadowRealm source evaluation

The 2026-10-07 dry implementation adds `PreparedScriptKind::ShadowRealmEvaluate`
to the existing finite-source compiler. It is distinct from `$262.evalScript`
and uses the indirect-eval environment owner: each invocation has fresh lexical
bindings, sloppy variables belong to the selected global environment, and strict
variables remain local. Source strictness comes from the evaluated text.
This follows [PerformShadowRealmEval and GetShadowRealmContext](https://tc39.es/proposal-shadowrealm/#sec-performshadowrealmeval).

Source candidate discovery includes acquired `evaluate` methods, aliases and
forwarded calls, and recursively compiles nested finite candidates. Discovery
never substitutes the actual acquired callee or receiver. Non-String operands
remain native validation failures without coercion. Unknown String values reach
the existing out-of-band dynamic-source rejection, now identified as
`ShadowRealmEvaluate` with ABI reason 10; JavaScript catch and wrapper error
mapping cannot turn that capability boundary into an ordinary exception.

Optional forwarding now uses the same candidate registrar as ordinary calls.
A temporary syntax projection retains property and call structure so both
`method?.call(...)` and `method.call?.(...)`, including finite-array `apply`,
keep their original forwarded argument positions. The projection is consumed
only by finite-source discovery. It retains source spans for function identity,
borrows real function plans, and updates bounded compilation hints; it never
enters pointer-keyed environment/continuation lowering or supplies runtime
callee authority. The original optional chain still owns its Get, receiver,
argument evaluation, short circuit and Call. The expanded controls cover
source-function parameter ownership, nested Function imports, custom methods,
getter order, argument-time replacement, skipped arguments and disabled loading.
Ordinary and optional discovery share the computed-key helper, including a
single retained text spelling from a named key; competing text spellings do
not gain a single-key candidate or alter the runtime key expression.
This follow-up passes the all-feature/all-target workspace check and the full
2,135-control IR suite in `tasks-dry-closure-foundation2`. The optional-alias
native cohort passes both modes in 364.18 seconds. `tasks-dry-closure-native1`
was deliberately stopped during its second case after 466 seconds (exit 143);
seven cases were incomplete or unstarted.

Shared Realm initialization now passes `tasks-realm-bootstrap1` in 210 seconds:
source guards, the all-feature/all-target workspace check (60.36 seconds;
Cargo: 59.02) and 16 focused AOT controls (six helper unit, ten integration),
with none failed or ignored. The validated optional artifact has main at
534,595 bytes, the initializer at 662,558 and ShadowRealm at 24,632; every body
is below 1 MiB and both callers share the helper. The earlier IR/cache/catalog
and native evidence keeps its preceding-source scope. `tasks-dry-closure-native2`
received an unexpected SIGTERM after 188 seconds (exit 143), during the first
strict-mode compile: sloppy execution passed, but no test function completed.
Service accounting recorded 3.8 GiB peak and no OOM termination entry.
`tasks-dry-closure-native3` passed all six queued ShadowRealm gates before a later
SIGTERM ended the run at 5,153 seconds (30 passing, four failing, 19 incomplete
or unstarted; 4 GiB service peak, no OOM termination entry). The unrelated global
Reference and class-identity repairs are written; their checkpoint and the
remaining native queue are pending under the same CPU/RAM limits. The optional
sloppy module was 41,458,110 bytes with observed compilation at 75.157/72.615
seconds versus the earlier 160.616/140.921, under different cache histories.
See the [runtime performance profile](runtime-performance-profile.md).

`emit_evaluate_shadow_realm_source` requires the native caller's validated
receiver Realm and String value. It shares dispatch with existing prepared
global scripts but has a closed, separate dispatch owner. A deferred parse or
early error is created in the active caller Realm, before entering the target,
and leaves `source_parsed` false. Executable source sets that flag and enters the
receiver Realm; normal and thrown completions return only after the caller Realm
is restored. The native boundary consumes the flag to preserve source
SyntaxErrors while applying wrapped-value and target-error rules to execution.
All Realm, environment and completion values use existing rooted GC locals.

Authored controls in `lila-ir/src/tests/prepared_eval.rs` cover distinct units,
aliases, nested candidates, strict variable ownership and deferred early errors.
`aot_shadow_realm_prepared_evaluate` covers repeated lexical declarations,
retained lexical closures, strict/sloppy variable lifetimes, explicit receiver
Realm selection, String-only input, caller parse errors, target throws and an
unmatched source's typed rejection. The IR controls and all-target workspace
check pass. `tasks-shadow-native2` passes the selected construction/isolation,
fresh-lexical and strictness-lifetime, and wrapped-callable controls in both
strictness modes, plus the module cache/isolation control. The remaining authored
native cohorts, shared-initializer native acceptance, T24 acceptance and pinned
results remain open. Native wrapper/importValue integration retains
prepared parse products while host discovery closes nested module requests
before artifact identity. Capped verification remains required by the
[implementation sequence](shadowrealm-implementation-sequence.md).
