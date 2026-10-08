# Test262 execution follows the original source

Source update — 2026-10-07. The harness no longer decides compiler support from
a test filename or its declared feature list. The former immutable ArrayBuffer,
SharedArrayBuffer and four Proxy/Realm path filters are retired together with
their positive path allowlists. This also removes the supervisor's metadata-only
execution mode and both early Unsupported result producers.

Every nonempty requested execution inventory needs a selected case worker.
Only that exact single-case worker role, or the explicit test-fixture role,
can enter the compiler in process. An absent worker cannot become permission to
classify cases without compiling them. Empty or fully resumed inventories keep
their existing completed-checkpoint behavior; no program remains to execute.

Materialization preserves the original program and its declared helpers. Parsing,
early errors, lowering, module resolution, code generation and the actual runtime
own their normal typed diagnostics. Compile-negative success still requires the
correct original phase and error category. Runtime-negative success still needs
an actual JavaScript exception. Dynamic-source rejection, unavailable weak
reachability, worker timeout and traps keep their original explicit failed
domains and cannot be caught or relabeled as a passing expected exception.

The case cache scheduler may prioritize compile-negative or already-cached work;
it no longer infers a cheap unsupported execution from paths or feature names.
Source-preservation and native semantic controls stay in place. Tests that only
asserted the retired path classifier's answers are removed. New focused controls
cover mandatory worker ownership for formerly rejected metadata, original parse
diagnostics and wrong-phase negatives, actual runtime completion under the
former Proxy path plus both former feature filters, and an actual weak-reachability
rejection that cannot satisfy an expected JavaScript TypeError.

The exact selector ownership TSV removes the three retired declarations.
The generated shortcut inventory has deliberately not been rewritten during
the no-compilation source phase: its official source scanner invokes rustc.
After the full dry batch, refresh it with the existing generator and inspect the
result before committing it:

```sh
./scripts/audit-test262-shortcuts.sh --target crates/lila-test262/src/lib.rs > target/shortcut-inventory.next.md
```

Only a successful generator result may replace
`test262/backlog/shortcut-inventory.md`; then run the ordinary `--check` gate.
No conformance count, generated Test262 status or PASS receipt changes with this
source retirement. Compilation, the focused controls and the later original
suite executions remain required before acceptance.

The authored source-routing controls run after the combined type checkpoint:

```sh
python3 scripts/limited_verification.py --memory-mib 4096 -- cargo test --locked -p lila-test262 --features spec-exec-oracle --lib source_execution_routing_tests
python3 scripts/limited_verification.py --memory-mib 4096 -- cargo test --locked -p lila-test262 --features spec-exec-oracle --lib differential::test262_seeds::tests::feature_annotations_and_async_agent_hosts_keep_the_original_runner_owners -- --exact
```

The existing `tests::default_execution_refuses_a_missing_supervisor_before_admission`,
`tests::run_one_case_rejects_early_negative_when_parser_reports_parse_phase`,
`tests::runtime_dynamic_source_cannot_pass_by_catching_or_matching_a_js_error`,
and `tests::every_typed_dynamic_source_diagnostic_is_accounted_as_unsupported`
remain relevant joined-checkpoint controls. Their source paths and typed
classification owners were inspected; no test execution is implied.
