use std::fs;
use std::path::{Path, PathBuf};

const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const DELEGATION_SOURCE: &str = include_str!("../src/generator_delegation.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/async-generator-body-status-word.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .replace(",)", ")")
        .replace(",]", "]")
}

fn normalized_code(source: &str) -> String {
    normalized(
        &source
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn unique_position(body: &str, needle: &str, label: &str) -> usize {
    assert_eq!(
        body.matches(needle).count(),
        1,
        "{label} must occur exactly once"
    );
    body.find(needle)
        .unwrap_or_else(|| panic!("missing sentinel: {label}"))
}

fn collect_rust_sources(directory: &Path, sources: &mut Vec<(PathBuf, String)>) {
    let mut paths = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        .map(|entry| entry.expect("failed to read source entry").path())
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        if path.is_dir() {
            collect_rust_sources(&path, sources);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            sources.push((path, source));
        }
    }
}

#[test]
fn body_status_is_the_exact_six_value_backend_domain() {
    let declaration = bounded(
        HEAP_SOURCE,
        "pub(crate) enum AsyncGeneratorBodyStatus {",
        "}\n\nimpl AsyncGeneratorBodyStatus {",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "Idle,",
            "Running,",
            "Await,",
            "Yield,",
            "Complete,",
            "Throw,"
        ],
    );

    let policy = normalized_code(bounded(
        HEAP_SOURCE,
        "impl AsyncGeneratorBodyStatus {",
        "/// One strictly validated snapshot of an async-generator body status.",
    ));
    assert_eq!(
        policy,
        normalized_code(
            r#"
            const ALL: [Self; 6] = [
                Self::Idle,
                Self::Running,
                Self::Await,
                Self::Yield,
                Self::Complete,
                Self::Throw,
            ];

            const fn word(self) -> u64 {
                match self {
                    Self::Idle => 0,
                    Self::Running => 1,
                    Self::Await => 2,
                    Self::Yield => 3,
                    Self::Complete => 4,
                    Self::Throw => 5,
                }
            }
        }
            "#,
        )
    );
    assert_eq!(policy.matches("=>").count(), 6);
    assert!(!policy.contains("_=>"));
    assert!(!policy.contains("unreachable!"));

    let domain = bounded(
        HEAP_SOURCE,
        "/// The closed backend status stored around an async-generator body invocation.",
        "pub(crate) const GENERATOR_RESUME_STATE_INITIALIZING",
    );
    assert!(!domain.contains("repr("));
    assert!(!HEAP_SOURCE.contains("impl Default for AsyncGeneratorBodyStatus"));
    assert!(!HEAP_SOURCE.contains("impl From<u64> for AsyncGeneratorBodyStatus"));
    assert!(!HEAP_SOURCE.contains("impl From<i64> for AsyncGeneratorBodyStatus"));
    assert!(!HEAP_SOURCE.contains("impl From<bool> for AsyncGeneratorBodyStatus"));
    for retired_constant in [
        "ASYNC_GENERATOR_BODY_STATUS_IDLE",
        "ASYNC_GENERATOR_BODY_STATUS_RUNNING",
        "ASYNC_GENERATOR_BODY_STATUS_AWAIT",
        "ASYNC_GENERATOR_BODY_STATUS_YIELD",
        "ASYNC_GENERATOR_BODY_STATUS_COMPLETE",
        "ASYNC_GENERATOR_BODY_STATUS_THROW",
    ] {
        assert!(
            !HEAP_SOURCE.contains(retired_constant),
            "{retired_constant}"
        );
    }

    let token = bounded(
        HEAP_SOURCE,
        "/// One strictly validated snapshot of an async-generator body status.",
        "pub(crate) const GENERATOR_RESUME_STATE_INITIALIZING",
    );
    assert!(token.contains("#[must_use"));
    assert_eq!(
        token
            .matches("pub(crate) struct LoadedAsyncGeneratorBodyStatus(u32);")
            .count(),
        1
    );
    assert!(!token.contains("#[derive(Clone"));
    assert!(!token.contains("#[derive(Copy"));
    assert!(!HEAP_SOURCE.contains("impl LoadedAsyncGeneratorBodyStatus"));
    assert!(!HEAP_SOURCE.contains("Deref for LoadedAsyncGeneratorBodyStatus"));
    assert_eq!(
        HEAP_SOURCE
            .matches("LoadedAsyncGeneratorBodyStatus(")
            .count(),
        2,
        "only the tuple declaration and strict loader may construct the token"
    );
}

#[test]
fn body_status_heap_boundary_is_private_strict_and_opaque() {
    assert_eq!(
        HEAP_SOURCE
            .matches("const HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET: u64 = 144;")
            .count(),
        1
    );
    assert!(!HEAP_SOURCE.contains("pub(crate) const HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET"));
    assert_eq!(
        HEAP_SOURCE
            .matches("HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET")
            .count(),
        4,
        "only declaration, layout, typed store and strict load may name the raw offset"
    );
    for source in [
        FUNCTIONS_SOURCE,
        PROMISE_SOURCE,
        STANDARD_SOURCE,
        CONTROL_FLOW_SOURCE,
        DELEGATION_SOURCE,
    ] {
        assert!(!source.contains("HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET"));
    }

    let store = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_store_async_generator_body_status(",
        "/// Load and strictly validate one async-generator body-status snapshot.",
    );
    assert!(store.contains("status: AsyncGeneratorBodyStatus,"));
    assert_eq!(
        store
            .matches("HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET")
            .count(),
        1
    );
    assert_eq!(store.matches("status.word()").count(), 1);

    let loader = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_load_async_generator_body_status_strict(",
        "/// Emit one comparison against a strictly loaded body-status word.",
    );
    assert!(loader.contains(") -> LoadedAsyncGeneratorBodyStatus {"));
    assert_eq!(loader.matches("reserve_temp_local()").count(), 1);
    assert_eq!(
        loader
            .matches("HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET")
            .count(),
        1
    );
    assert_eq!(
        loader
            .matches("for status in AsyncGeneratorBodyStatus::ALL")
            .count(),
        1
    );
    assert_eq!(loader.matches("status.word()").count(), 1);
    assert_eq!(loader.matches("Instruction::Else").count(), 1);
    assert_eq!(loader.matches("Instruction::Unreachable").count(), 1);
    assert_eq!(
        loader
            .matches("LoadedAsyncGeneratorBodyStatus(status_word_local)")
            .count(),
        1
    );
    assert!(!loader.contains("_ =>"));

    let comparison = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_async_generator_body_status_equals(",
        "/// Release the private local owned by a body-status snapshot.",
    );
    assert!(comparison.contains("loaded: &LoadedAsyncGeneratorBodyStatus,"));
    assert!(comparison.contains("expected: AsyncGeneratorBodyStatus,"));
    assert_eq!(comparison.matches("LocalGet(loaded.0)").count(), 1);
    assert_eq!(comparison.matches("expected.word()").count(), 1);

    let release = bounded(
        HEAP_SOURCE,
        "pub(crate) fn release_loaded_async_generator_body_status(",
        "/// Initialize a Promise record in the sole valid non-terminal state.",
    );
    assert!(release.contains("loaded: LoadedAsyncGeneratorBodyStatus,"));
    assert!(!release.contains("&LoadedAsyncGeneratorBodyStatus"));
    assert_eq!(release.matches("release_temp_local(loaded.0)").count(), 1);
}

#[test]
fn body_status_has_exactly_fifteen_writers_and_three_readers() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);

    let mut totals = [0; 5];
    for (path, source) in sources {
        let relative = path
            .strip_prefix(&source_root)
            .expect("collected source must remain below src")
            .to_string_lossy();
        let expected = match relative.as_ref() {
            "heap.rs" => (4, 1, 1, 1, 1),
            "functions.rs" => (0, 5, 1, 2, 1),
            "builtins/promise.rs" => (0, 1, 2, 2, 2),
            "builtins/standard.rs" => (0, 1, 0, 0, 0),
            "control_flow.rs" => (0, 5, 0, 0, 0),
            "generator_delegation.rs" => (0, 3, 0, 0, 0),
            _ => (0, 0, 0, 0, 0),
        };
        let actual = (
            source
                .matches("HEAP_ASYNC_GENERATOR_BODY_STATUS_OFFSET")
                .count(),
            source
                .matches("emit_store_async_generator_body_status(")
                .count(),
            source
                .matches("emit_load_async_generator_body_status_strict(")
                .count(),
            source
                .matches("emit_async_generator_body_status_equals(")
                .count(),
            source
                .matches("release_loaded_async_generator_body_status(")
                .count(),
        );
        assert_eq!(
            actual, expected,
            "unexpected async-generator body-status owner in {relative}"
        );
        totals[0] += actual.0;
        totals[1] += actual.1;
        totals[2] += actual.2;
        totals[3] += actual.3;
        totals[4] += actual.4;
    }
    assert_eq!(totals, [4, 16, 4, 5, 4]);

    for (source, expected) in [
        (
            FUNCTIONS_SOURCE,
            [
                ("Idle", 1),
                ("Running", 2),
                ("Await", 0),
                ("Yield", 1),
                ("Complete", 2),
                ("Throw", 1),
            ],
        ),
        (
            PROMISE_SOURCE,
            [
                ("Idle", 0),
                ("Running", 0),
                ("Await", 3),
                ("Yield", 0),
                ("Complete", 0),
                ("Throw", 0),
            ],
        ),
        (
            STANDARD_SOURCE,
            [
                ("Idle", 0),
                ("Running", 0),
                ("Await", 1),
                ("Yield", 0),
                ("Complete", 0),
                ("Throw", 0),
            ],
        ),
        (
            CONTROL_FLOW_SOURCE,
            [
                ("Idle", 0),
                ("Running", 0),
                ("Await", 4),
                ("Yield", 1),
                ("Complete", 0),
                ("Throw", 0),
            ],
        ),
        (
            DELEGATION_SOURCE,
            [
                ("Idle", 0),
                ("Running", 0),
                ("Await", 2),
                ("Yield", 1),
                ("Complete", 0),
                ("Throw", 0),
            ],
        ),
    ] {
        let source = normalized_code(source);
        for (variant, count) in expected {
            let variant_name = format!("AsyncGeneratorBodyStatus::{variant}");
            assert_eq!(
                source.matches(variant_name.as_str()).count(),
                count,
                "unexpected {variant} body-status owner count"
            );
        }
    }
}

#[test]
fn body_status_routes_preserve_publication_and_snapshot_order() {
    let allocation = normalized_code(bounded(
        FUNCTIONS_SOURCE,
        "if can_call_async_generator {",
        "if can_call_async {",
    ));
    let activation_allocation = unique_position(
        &allocation,
        "self.emit_heap_alloc_const(HEAP_ASYNC_GENERATOR_ACTIVATION_RECORD_SIZE,function)?;",
        "async-generator activation allocation",
    );
    let idle = unique_position(
        &allocation,
        "self.emit_store_async_generator_body_status(async_generator_activation_local,AsyncGeneratorBodyStatus::Idle,function);",
        "typed Idle initialization",
    );
    let suspended_start = unique_position(
        &allocation,
        "self.emit_store_async_generator_execution_state(async_generator_activation_local,AsyncGeneratorExecutionState::SuspendedStart,function);",
        "typed suspended-start initialization",
    );
    let activation_publication = unique_position(
        &allocation,
        "self.store_i64_local_at_offset(payload_local,HEAP_ASYNC_GENERATOR_ACTIVATION_OFFSET,async_generator_activation_local,function);",
        "activation publication",
    );
    assert!(activation_allocation < idle);
    assert!(idle < suspended_start && suspended_start < activation_publication);

    let body_driver = normalized_code(bounded(
        FUNCTIONS_SOURCE,
        "pub(crate) fn emit_start_async_generator_body(",
        "pub(crate) fn emit_load_function_flags(",
    ));
    assert!(body_driver.contains("letresume_state_local=self.reserve_temp_local();"));
    assert!(!body_driver.contains("body_status_local"));
    let running_store = unique_position(
        &body_driver,
        "self.emit_store_async_generator_body_status(activation_local,AsyncGeneratorBodyStatus::Running,function);",
        "body-entry Running store",
    );
    let body_call = unique_position(
        &body_driver,
        "Instruction::CallIndirect",
        "async-generator body call",
    );
    let strict_load = unique_position(
        &body_driver,
        "self.emit_load_async_generator_body_status_strict(activation_local,function)",
        "strict body-status snapshot",
    );
    let yield_route = unique_position(
        &body_driver,
        "AsyncGeneratorBodyStatus::Yield",
        "Yield route",
    );
    let running_route = body_driver
        .rfind("AsyncGeneratorBodyStatus::Running")
        .expect("missing Running completion route");
    let release = unique_position(
        &body_driver,
        "self.release_loaded_async_generator_body_status(body_status);",
        "body-status token release",
    );
    assert!(running_store < body_call && body_call < strict_load);
    assert!(strict_load < yield_route && yield_route < running_route && running_route < release);
    assert_eq!(
        body_driver
            .matches("AsyncGeneratorBodyStatus::Complete")
            .count(),
        2
    );
    assert_eq!(
        body_driver
            .matches("AsyncGeneratorBodyStatus::Throw")
            .count(),
        1
    );

    for (start, end) in [
        (
            "fn emit_run_async_generator_await_job(",
            "fn emit_run_async_generator_await_return_job(",
        ),
        (
            "fn emit_run_async_generator_yield_return_job(",
            "fn emit_run_async_generator_yield_job(",
        ),
    ] {
        let owner = normalized_code(bounded(PROMISE_SOURCE, start, end));
        let execution_load = unique_position(
            &owner,
            "self.emit_load_async_generator_execution_state_strict(activation_local,function)",
            "strict execution-state load",
        );
        let executing = unique_position(
            &owner,
            "AsyncGeneratorExecutionState::Executing",
            "Executing assertion",
        );
        let body_load = unique_position(
            &owner,
            "self.emit_load_async_generator_body_status_strict(activation_local,function)",
            "strict body-status load",
        );
        let await_route =
            unique_position(&owner, "AsyncGeneratorBodyStatus::Await", "Await assertion");
        let resume = unique_position(
            &owner,
            "self.emit_start_async_generator_body(activation_local,function)?;",
            "body resumption",
        );
        let body_release = unique_position(
            &owner,
            "self.release_loaded_async_generator_body_status(body_status);",
            "body-status release",
        );
        let execution_release = unique_position(
            &owner,
            "self.release_loaded_async_generator_execution_state(execution_state);",
            "execution-state release",
        );
        assert!(execution_load < executing && executing < body_load);
        assert!(body_load < await_route && await_route < resume);
        assert!(resume < body_release && body_release < execution_release);
    }
}

#[test]
fn contract_records_scope_owner_census_and_verification() {
    let contract = normalized(CONTRACT);
    assert!(contract.contains("six-valueasync-generatorbody-statusdomain"));
    assert!(contract.contains("fifteenproductwritersandthreeproductreaders"));
    assert!(contract.contains("deliberatelydistinct"));
    assert!(contract.contains("`AsyncGeneratorExecutionState`domain"));
    assert!(
        contract.contains("body-status,execution-state,request-completionandawait-usingstructure")
    );
    assert!(contract.contains("eachpass`5/5`"));
}
