use std::fs;
use std::path::{Path, PathBuf};

const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const DELEGATION_SOURCE: &str = include_str!("../src/generator_delegation.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/async-generator-execution-state-word.md");

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

fn positions(body: &str, needle: &str) -> Vec<usize> {
    body.match_indices(needle).map(|(index, _)| index).collect()
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

fn async_generator_builtin_owner() -> &'static str {
    bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::AsyncGeneratorPrototypeNext\n            | StandardBuiltinId::AsyncGeneratorPrototypeReturn\n            | StandardBuiltinId::AsyncGeneratorPrototypeThrow => {",
        "StandardBuiltinId::ArrayIteratorNext => {",
    )
}

#[test]
fn execution_state_is_the_exact_five_value_ecmascript_domain() {
    let declaration = bounded(
        HEAP_SOURCE,
        "pub(crate) enum AsyncGeneratorExecutionState {",
        "}\n\nimpl AsyncGeneratorExecutionState {",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "SuspendedStart,",
            "SuspendedYield,",
            "Executing,",
            "DrainingQueue,",
            "Completed,",
        ],
        "the persisted domain must match ECMA-262 [[AsyncGeneratorState]] exactly"
    );

    let policy = normalized_code(bounded(
        HEAP_SOURCE,
        "impl AsyncGeneratorExecutionState {",
        "/// One strictly validated snapshot of an async-generator execution state.",
    ));
    assert_eq!(
        policy,
        normalized_code(
            r#"
            const ALL: [Self; 5] = [
                Self::SuspendedStart,
                Self::SuspendedYield,
                Self::Executing,
                Self::DrainingQueue,
                Self::Completed,
            ];

            const fn word(self) -> u64 {
                match self {
                    Self::SuspendedStart => 0,
                    Self::SuspendedYield => 1,
                    Self::Executing => 2,
                    Self::DrainingQueue => 3,
                    Self::Completed => 4,
                }
            }
        }
            "#,
        ),
        "execution state must retain one complete list and one exhaustive stable projection"
    );
    assert_eq!(policy.matches("=>").count(), 5);
    assert!(!policy.contains("_=>"));
    assert!(!policy.contains("unreachable!"));

    let domain = bounded(
        HEAP_SOURCE,
        "/// The closed `[[AsyncGeneratorState]]` lifecycle stored in an activation.",
        "pub(crate) const GENERATOR_RESUME_STATE_INITIALIZING",
    );
    assert!(!domain.contains("repr("));
    assert!(!domain.contains("SuspendedAwait"));
    assert!(!HEAP_SOURCE.contains("impl Default for AsyncGeneratorExecutionState"));
    assert!(!HEAP_SOURCE.contains("impl From<u64> for AsyncGeneratorExecutionState"));
    assert!(!HEAP_SOURCE.contains("impl From<i64> for AsyncGeneratorExecutionState"));
    assert!(!HEAP_SOURCE.contains("impl From<bool> for AsyncGeneratorExecutionState"));
    assert!(!HEAP_SOURCE.contains("ASYNC_GENERATOR_STATE_"));

    let token = bounded(
        HEAP_SOURCE,
        "/// One strictly validated snapshot of an async-generator execution state.",
        "pub(crate) const GENERATOR_RESUME_STATE_INITIALIZING",
    );
    assert!(token.contains("#[must_use"));
    assert_eq!(
        token
            .matches("pub(crate) struct LoadedAsyncGeneratorExecutionState(u32);")
            .count(),
        1
    );
    assert!(!token.contains("#[derive(Clone"));
    assert!(!token.contains("#[derive(Copy"));
    assert!(!HEAP_SOURCE.contains("impl LoadedAsyncGeneratorExecutionState"));
    assert!(!HEAP_SOURCE.contains("Deref for LoadedAsyncGeneratorExecutionState"));
    assert_eq!(
        HEAP_SOURCE
            .matches("LoadedAsyncGeneratorExecutionState(")
            .count(),
        2,
        "only the tuple declaration and strict loader may construct the token"
    );
}

#[test]
fn execution_state_heap_boundary_is_private_strict_and_opaque() {
    assert_eq!(
        HEAP_SOURCE
            .matches("const HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET: u64 = 24;")
            .count(),
        1
    );
    assert!(!HEAP_SOURCE.contains("pub(crate) const HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET"));
    assert_eq!(
        HEAP_SOURCE
            .matches("HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET")
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
        assert!(!source.contains("HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET"));
    }

    let store = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_store_async_generator_execution_state(",
        "/// Load and strictly validate one async-generator execution-state snapshot.",
    );
    assert!(store.contains("state: AsyncGeneratorExecutionState,"));
    assert_eq!(
        store
            .matches("HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET")
            .count(),
        1
    );
    assert_eq!(store.matches("state.word()").count(), 1);

    let loader = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_load_async_generator_execution_state_strict(",
        "/// Emit one comparison against a strictly loaded execution-state word.",
    );
    assert!(loader.contains(") -> LoadedAsyncGeneratorExecutionState {"));
    assert_eq!(loader.matches("reserve_temp_local()").count(), 1);
    assert_eq!(
        loader
            .matches("HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET")
            .count(),
        1
    );
    assert_eq!(
        loader
            .matches("for state in AsyncGeneratorExecutionState::ALL")
            .count(),
        1
    );
    assert_eq!(loader.matches("state.word()").count(), 1);
    assert_eq!(loader.matches("Instruction::Else").count(), 1);
    assert_eq!(loader.matches("Instruction::Unreachable").count(), 1);
    assert_eq!(
        loader
            .matches("LoadedAsyncGeneratorExecutionState(state_word_local)")
            .count(),
        1
    );
    assert!(!loader.contains("_ =>"));

    let comparison = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_async_generator_execution_state_equals(",
        "/// Release the private local owned by an execution-state snapshot.",
    );
    assert!(comparison.contains("loaded: &LoadedAsyncGeneratorExecutionState,"));
    assert!(comparison.contains("expected: AsyncGeneratorExecutionState,"));
    assert_eq!(comparison.matches("LocalGet(loaded.0)").count(), 1);
    assert_eq!(comparison.matches("expected.word()").count(), 1);

    let release = bounded(
        HEAP_SOURCE,
        "pub(crate) fn release_loaded_async_generator_execution_state(",
        "/// Store one status from the closed async-generator body domain.",
    );
    assert!(release.contains("loaded: LoadedAsyncGeneratorExecutionState,"));
    assert!(!release.contains("&LoadedAsyncGeneratorExecutionState"));
    assert_eq!(release.matches("release_temp_local(loaded.0)").count(), 1);
}

#[test]
fn execution_state_has_exactly_seventeen_writers_and_three_readers() {
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
            "functions.rs" => (0, 4, 0, 0, 0),
            "builtins/promise.rs" => (0, 2, 2, 2, 2),
            "builtins/standard.rs" => (0, 3, 1, 3, 1),
            "control_flow.rs" => (0, 5, 0, 0, 0),
            "generator_delegation.rs" => (0, 3, 0, 0, 0),
            _ => (0, 0, 0, 0, 0),
        };
        let actual = (
            source
                .matches("HEAP_ASYNC_GENERATOR_EXECUTION_STATE_OFFSET")
                .count(),
            source
                .matches("emit_store_async_generator_execution_state(")
                .count(),
            source
                .matches("emit_load_async_generator_execution_state_strict(")
                .count(),
            source
                .matches("emit_async_generator_execution_state_equals(")
                .count(),
            source
                .matches("release_loaded_async_generator_execution_state(")
                .count(),
        );
        assert_eq!(
            actual, expected,
            "unexpected async-generator execution-state owner in {relative}"
        );
        totals[0] += actual.0;
        totals[1] += actual.1;
        totals[2] += actual.2;
        totals[3] += actual.3;
        totals[4] += actual.4;
    }
    assert_eq!(totals, [4, 18, 4, 6, 4]);

    for (source, expected) in [
        (
            FUNCTIONS_SOURCE,
            [
                ("SuspendedStart", 1),
                ("SuspendedYield", 0),
                ("Executing", 1),
                ("DrainingQueue", 2),
                ("Completed", 0),
            ],
        ),
        (
            PROMISE_SOURCE,
            [
                ("SuspendedStart", 0),
                ("SuspendedYield", 0),
                ("Executing", 3),
                ("DrainingQueue", 0),
                ("Completed", 1),
            ],
        ),
        (
            STANDARD_SOURCE,
            [
                ("SuspendedStart", 1),
                ("SuspendedYield", 1),
                ("Executing", 1),
                ("DrainingQueue", 2),
                ("Completed", 1),
            ],
        ),
        (
            CONTROL_FLOW_SOURCE,
            [
                ("SuspendedStart", 0),
                ("SuspendedYield", 1),
                ("Executing", 4),
                ("DrainingQueue", 0),
                ("Completed", 0),
            ],
        ),
        (
            DELEGATION_SOURCE,
            [
                ("SuspendedStart", 0),
                ("SuspendedYield", 1),
                ("Executing", 2),
                ("DrainingQueue", 0),
                ("Completed", 0),
            ],
        ),
    ] {
        let source = normalized_code(source);
        for (variant, count) in expected {
            let variant_name = format!("AsyncGeneratorExecutionState::{variant}");
            assert_eq!(
                source.matches(variant_name.as_str()).count(),
                count,
                "unexpected {variant} execution-state owner count"
            );
        }
    }
}

#[test]
fn lifecycle_owners_preserve_state_transition_order() {
    let allocation = normalized_code(bounded(
        FUNCTIONS_SOURCE,
        "if can_call_async_generator {",
        "if can_call_async {",
    ));
    assert_eq!(
        allocation
            .matches("emit_store_async_generator_execution_state(")
            .count(),
        1
    );
    let activation_allocation = unique_position(
        &allocation,
        "self.emit_heap_alloc_const(HEAP_ASYNC_GENERATOR_ACTIVATION_RECORD_SIZE,function)?;",
        "async-generator activation allocation",
    );
    let last_private_initialization = unique_position(
        &allocation,
        "HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET",
        "last generic activation field initialization",
    );
    let suspended_start = unique_position(
        &allocation,
        "self.emit_store_async_generator_execution_state(async_generator_activation_local,AsyncGeneratorExecutionState::SuspendedStart,function);",
        "typed suspended-start publication",
    );
    let activation_publication = unique_position(
        &allocation,
        "self.store_i64_local_at_offset(payload_local,HEAP_ASYNC_GENERATOR_ACTIVATION_OFFSET,async_generator_activation_local,function);",
        "activation publication through the generator object",
    );
    assert!(activation_allocation < last_private_initialization);
    assert!(last_private_initialization < suspended_start);
    assert!(suspended_start < activation_publication);

    let body_driver = normalized_code(bounded(
        FUNCTIONS_SOURCE,
        "pub(crate) fn emit_start_async_generator_body(",
        "pub(crate) fn emit_load_function_flags(",
    ));
    assert_eq!(
        body_driver
            .matches("AsyncGeneratorExecutionState::Executing")
            .count(),
        1
    );
    assert_eq!(
        body_driver
            .matches("AsyncGeneratorExecutionState::DrainingQueue")
            .count(),
        2
    );
    let executing = unique_position(
        &body_driver,
        "AsyncGeneratorExecutionState::Executing",
        "body entry executing state",
    );
    let running = unique_position(
        &body_driver,
        "self.emit_store_async_generator_body_status(activation_local,AsyncGeneratorBodyStatus::Running,function);",
        "body running status",
    );
    let body_call = unique_position(
        &body_driver,
        "Instruction::CallIndirect",
        "async-generator body call",
    );
    let draining = positions(&body_driver, "AsyncGeneratorExecutionState::DrainingQueue");
    assert!(executing < running && running < body_call);
    assert_eq!(draining.len(), 2);
    assert!(body_call < draining[0] && draining[0] < draining[1]);

    let drain = normalized_code(bounded(
        PROMISE_SOURCE,
        "pub(crate) fn emit_drain_async_generator_queue(",
        "fn emit_run_async_generator_await_job(",
    ));
    let active_clear = unique_position(
        &drain,
        "self.store_i64_const_at_offset(activation_local,HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,0,function);",
        "empty-queue active-request clear",
    );
    let completed = unique_position(
        &drain,
        "self.emit_store_async_generator_execution_state(activation_local,AsyncGeneratorExecutionState::Completed,function);",
        "empty-queue completed state",
    );
    assert!(active_clear < completed);

    let builtin = normalized_code(async_generator_builtin_owner());
    assert_eq!(
        builtin
            .matches("emit_load_async_generator_execution_state_strict(")
            .count(),
        1
    );
    assert_eq!(
        builtin
            .matches("emit_async_generator_execution_state_equals(")
            .count(),
        3
    );
    let tail_publication = unique_position(
        &builtin,
        "self.store_i64_local_at_offset(activation_local,HEAP_ASYNC_GENERATOR_QUEUE_TAIL_OFFSET,request_local,function);",
        "request queue-tail publication",
    );
    let load = unique_position(
        &builtin,
        "self.emit_load_async_generator_execution_state_strict(activation_local,function)",
        "strict execution-state snapshot",
    );
    let completed_route = unique_position(
        &builtin,
        "AsyncGeneratorExecutionState::Completed",
        "completed request route",
    );
    let suspended_yield_route = unique_position(
        &builtin,
        "AsyncGeneratorExecutionState::SuspendedYield",
        "suspended-yield request route",
    );
    let suspended_start_route = unique_position(
        &builtin,
        "AsyncGeneratorExecutionState::SuspendedStart",
        "suspended-start request route",
    );
    let release = unique_position(
        &builtin,
        "self.release_loaded_async_generator_execution_state(execution_state);",
        "execution-state token release",
    );
    assert!(tail_publication < load && load < completed_route);
    assert!(
        completed_route < suspended_yield_route && suspended_yield_route < suspended_start_route
    );
    assert!(suspended_start_route < release);
}

#[test]
fn await_and_yield_paths_keep_execution_state_distinct_from_body_status() {
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
        let load = unique_position(
            &owner,
            "self.emit_load_async_generator_execution_state_strict(activation_local,function)",
            "strict await-job state load",
        );
        let executing = unique_position(
            &owner,
            "AsyncGeneratorExecutionState::Executing",
            "await-job executing assertion",
        );
        let trap = owner
            .find("function.instruction(&Instruction::Unreachable);")
            .expect("await-job state invariant trap must exist");
        let body_status_load = unique_position(
            &owner,
            "self.emit_load_async_generator_body_status_strict(activation_local,function)",
            "strict body-status snapshot",
        );
        let body_status_await = unique_position(
            &owner,
            "AsyncGeneratorBodyStatus::Await",
            "separate Await body-status route",
        );
        let resume = unique_position(
            &owner,
            "self.emit_start_async_generator_body(activation_local,function)?;",
            "await-job body resumption",
        );
        let release = unique_position(
            &owner,
            "self.release_loaded_async_generator_execution_state(execution_state);",
            "await-job state release",
        );
        let resume_kind_stores = positions(&owner, "self.emit_store_async_generator_resume_kind(");
        assert_eq!(resume_kind_stores.len(), 2);
        let first_temp_release = unique_position(
            &owner,
            "self.release_temp_local(queue_head_local);",
            "await-job first ordinary temporary release",
        );
        assert!(load < executing && executing < trap);
        assert!(
            trap < body_status_load
                && body_status_load < body_status_await
                && body_status_await < resume_kind_stores[0]
                && resume_kind_stores[0] < resume_kind_stores[1]
                && resume_kind_stores[1] < resume
                && resume < release
                && release < first_temp_release
        );
    }

    let yield_owner = normalized_code(bounded(
        PROMISE_SOURCE,
        "pub(crate) fn emit_complete_async_generator_yield(",
        "fn emit_run_promise_reaction_callback(",
    ));
    let return_await = unique_position(
        &yield_owner,
        "self.emit_async_generator_yield_return_reactions(",
        "yield Return Await setup",
    );
    let await_status = unique_position(
        &yield_owner,
        "AsyncGeneratorBodyStatus::Await",
        "yield Return body status",
    );
    let executing = unique_position(
        &yield_owner,
        "self.emit_store_async_generator_execution_state(activation_local,AsyncGeneratorExecutionState::Executing,function);",
        "yield Return executing state",
    );
    assert!(return_await < await_status && await_status < executing);

    let control_flow_mentions = CONTROL_FLOW_SOURCE
        .matches("emit_store_async_generator_execution_state(")
        .count();
    let delegation_mentions = DELEGATION_SOURCE
        .matches("emit_store_async_generator_execution_state(")
        .count();
    assert_eq!(control_flow_mentions, 5);
    assert_eq!(delegation_mentions, 3);
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("AsyncGeneratorExecutionState::Executing")
            .count(),
        4
    );
    assert_eq!(
        DELEGATION_SOURCE
            .matches("AsyncGeneratorExecutionState::Executing")
            .count(),
        2
    );
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("AsyncGeneratorExecutionState::SuspendedYield")
            .count(),
        1
    );
    assert_eq!(
        DELEGATION_SOURCE
            .matches("AsyncGeneratorExecutionState::SuspendedYield")
            .count(),
        1
    );

    assert!(CONTRACT.contains("exact five-value ECMA-262 domain"));
    assert!(CONTRACT.contains("word `5` is retired"));
    assert!(CONTRACT.contains("seventeen product writers and three product readers"));
    assert!(CONTRACT.contains("focused-verified"));
}
