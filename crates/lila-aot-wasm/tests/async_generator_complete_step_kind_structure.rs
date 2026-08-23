use std::fs;
use std::path::{Path, PathBuf};

const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const BUILTINS_SOURCE: &str = include_str!("../src/builtins/mod.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

const COMPLETE_STEP_CALL: &str = "self.emit_complete_async_generator_step(";

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

fn complete_step_helper() -> &'static str {
    bounded(
        PROMISE_SOURCE,
        "pub(crate) fn emit_complete_async_generator_step(",
        "pub(crate) fn emit_drain_async_generator_queue(",
    )
}

fn drain_owner() -> &'static str {
    bounded(
        PROMISE_SOURCE,
        "pub(crate) fn emit_drain_async_generator_queue(",
        "fn emit_run_async_generator_await_job(",
    )
}

fn await_return_owner() -> &'static str {
    bounded(
        PROMISE_SOURCE,
        "fn emit_run_async_generator_await_return_job(",
        "fn emit_run_async_generator_yield_return_job(",
    )
}

fn yield_owner() -> &'static str {
    bounded(
        PROMISE_SOURCE,
        "pub(crate) fn emit_complete_async_generator_yield(",
        "fn emit_run_promise_reaction_callback(",
    )
}

fn start_body_owner() -> &'static str {
    bounded(
        FUNCTIONS_SOURCE,
        "pub(crate) fn emit_start_async_generator_body(",
        "pub(crate) fn emit_load_function_flags(",
    )
}

fn standard_owner() -> &'static str {
    STANDARD_SOURCE
        .split_once("pub(crate) fn compile_standard_builtin(")
        .expect("compile_standard_builtin should exist")
        .1
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn normalized(source: &str) -> String {
    without_whitespace(source)
        .replace(",)", ")")
        .replace(",]", "]")
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

fn call_blocks(body: &str) -> Vec<&str> {
    body.match_indices(COMPLETE_STEP_CALL)
        .map(|(start, _)| {
            let call = &body[start..];
            let end = call
                .find(")?;")
                .expect("complete-step call should retain its fallible call boundary");
            &call[..end + 3]
        })
        .collect()
}

fn local_sequence<'a>(body: &'a str, prefix: &str, suffix: &str) -> Vec<&'a str> {
    body.lines()
        .filter_map(|line| line.trim().strip_prefix(prefix)?.strip_suffix(suffix))
        .collect()
}

fn collect_rust_sources(dir: &Path, sources: &mut Vec<(PathBuf, String)>) {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
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
fn complete_step_kind_is_one_closed_domain_with_one_boolean_projection() {
    let domain = bounded(
        PROMISE_SOURCE,
        "/// Whether an async-generator request publishes a yielded or terminal result.",
        "/// The original completion",
    );
    let declaration = bounded(
        PROMISE_SOURCE,
        "pub(crate) enum AsyncGeneratorCompleteStepKind {",
        "}\n\n/// The original completion",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        ["Yielded,", "Completed,"],
        "the complete-step result domain must remain exactly yielded or completed"
    );
    assert!(!declaration.contains("bool"));
    assert!(!domain.contains("Default"));
    assert!(!PROMISE_SOURCE.contains("impl Default for AsyncGeneratorCompleteStepKind"));
    assert!(!PROMISE_SOURCE.contains("impl AsyncGeneratorCompleteStepKind"));

    let helper = complete_step_helper();
    assert!(helper.contains("kind: AsyncGeneratorCompleteStepKind,"));
    assert!(!helper.contains("done: bool"));
    assert_eq!(helper.matches("let done = match kind {").count(), 1);
    assert_eq!(
        helper
            .matches("AsyncGeneratorCompleteStepKind::Yielded => false,")
            .count(),
        1
    );
    assert_eq!(
        helper
            .matches("AsyncGeneratorCompleteStepKind::Completed => true,")
            .count(),
        1
    );

    let projection = bounded(
        helper,
        "let done = match kind {",
        "};\n        self.emit_iterator_result_object_from_locals(",
    );
    assert_eq!(projection.matches("=>").count(), 2);
    assert!(!projection.contains("_ =>"));
    assert!(!projection.contains("unreachable!"));
    let materializer_call = concat!(
        "self.emit_iterator_result_object_from_locals(",
        "value_payload_local,value_tag_local,done,",
        "iterator_result_payload_local,iterator_result_tag_local,function)?;"
    );
    assert_eq!(
        normalized(helper).matches(materializer_call).count(),
        1,
        "the exhaustive lifecycle projection must be the iterator-result done argument"
    );

    let sources = [PROMISE_SOURCE, FUNCTIONS_SOURCE, STANDARD_SOURCE];
    assert_eq!(
        sources
            .iter()
            .map(|source| {
                source
                    .matches("AsyncGeneratorCompleteStepKind::Yielded")
                    .count()
            })
            .sum::<usize>(),
        2,
        "Yielded may appear only in the projection and the yield owner"
    );
    assert_eq!(
        sources
            .iter()
            .map(|source| {
                source
                    .matches("AsyncGeneratorCompleteStepKind::Completed")
                    .count()
            })
            .sum::<usize>(),
        11,
        "Completed may appear only in the projection and ten terminal calls"
    );

    assert_eq!(
        BUILTINS_SOURCE
            .matches("pub(crate) use promise::AsyncGeneratorCompleteStepKind;")
            .count(),
        1,
        "the private promise module must export the lifecycle state at crate visibility"
    );
    for (name, source) in [
        ("functions", FUNCTIONS_SOURCE),
        ("standard builtins", STANDARD_SOURCE),
    ] {
        assert!(
            source.contains("AsyncGeneratorCompleteStepKind::Completed"),
            "{name} must consume the crate-visible lifecycle state"
        );
        assert!(
            !source.contains("promise::AsyncGeneratorCompleteStepKind"),
            "{name} must not name the private promise module"
        );
    }
}

#[test]
fn complete_step_has_no_uninventoried_source_bypass() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);

    let mut helper_mentions = 0;
    let mut state_projections = 0;
    for (path, source) in sources {
        let relative = path
            .strip_prefix(&source_root)
            .expect("collected source must remain below src")
            .to_string_lossy();
        let expected_helper_mentions = match relative.as_ref() {
            "builtins/promise.rs" => 6,
            "functions.rs" | "builtins/standard.rs" => 3,
            _ => 0,
        };
        let actual_helper_mentions = source
            .matches("emit_complete_async_generator_step(")
            .count();
        assert_eq!(
            actual_helper_mentions, expected_helper_mentions,
            "unexpected async-generator complete-step definition or caller in {relative}"
        );
        assert_eq!(
            source
                .matches("::emit_complete_async_generator_step")
                .count(),
            0,
            "complete-step must not escape as an associated method item or UFCS call in {relative}"
        );
        helper_mentions += actual_helper_mentions;

        let expected_state_projections = match relative.as_ref() {
            "builtins/promise.rs" => 7,
            "functions.rs" | "builtins/standard.rs" => 3,
            _ => 0,
        };
        let actual_state_projections = source.matches("AsyncGeneratorCompleteStepKind::").count();
        assert_eq!(
            actual_state_projections, expected_state_projections,
            "unexpected async-generator complete-step state projection in {relative}"
        );
        state_projections += actual_state_projections;
    }

    assert_eq!(
        helper_mentions, 12,
        "the helper must have one definition and exactly eleven callers"
    );
    assert_eq!(
        state_projections, 13,
        "only the two exhaustive helper arms and eleven callers may project the state"
    );
}

#[test]
fn complete_step_call_owners_fix_the_ten_terminal_and_one_yielded_states() {
    let owners = [
        ("start body", start_body_owner(), 3, 0),
        ("drain queue", drain_owner(), 3, 0),
        ("await return", await_return_owner(), 1, 0),
        ("standard builtin", standard_owner(), 3, 0),
        ("complete yield", yield_owner(), 0, 1),
    ];

    let mut calls = 0;
    let mut completed = 0;
    let mut yielded = 0;
    for (name, owner, expected_completed, expected_yielded) in owners {
        let owner_calls = call_blocks(owner);
        assert_eq!(
            owner_calls.len(),
            expected_completed + expected_yielded,
            "{name} complete-step call count"
        );
        assert_eq!(
            owner
                .matches("AsyncGeneratorCompleteStepKind::Completed")
                .count(),
            expected_completed,
            "{name} terminal state count"
        );
        assert_eq!(
            owner
                .matches("AsyncGeneratorCompleteStepKind::Yielded")
                .count(),
            expected_yielded,
            "{name} yielded state count"
        );

        for call in owner_calls {
            let call = without_whitespace(call);
            assert!(!call.contains(",true,"), "{name} retained a raw true");
            assert!(!call.contains(",false,"), "{name} retained a raw false");
            assert_eq!(
                call.matches("AsyncGeneratorCompleteStepKind::").count(),
                1,
                "{name} must select exactly one lifecycle state per call"
            );
        }

        calls += expected_completed + expected_yielded;
        completed += expected_completed;
        yielded += expected_yielded;
    }

    assert_eq!((calls, completed, yielded), (11, 10, 1));
    assert_eq!(
        PROMISE_SOURCE.matches(COMPLETE_STEP_CALL).count()
            + FUNCTIONS_SOURCE.matches(COMPLETE_STEP_CALL).count()
            + STANDARD_SOURCE.matches(COMPLETE_STEP_CALL).count(),
        calls,
        "every complete-step call must belong to one of the five reviewed owners"
    );
}

#[test]
fn complete_step_preserves_request_settlement_order_and_temp_lifetime() {
    let helper = complete_step_helper();
    let normalized_helper = normalized(helper);

    let active_load = concat!(
        "self.load_i64_to_local_from_offset(activation_local,",
        "HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,request_local,function);"
    );
    let capability_wiring = concat!(
        "for(offset,destination_local)in[",
        "(HEAP_ASYNC_GENERATOR_REQUEST_PROMISE_PAYLOAD_OFFSET,promise_payload_local),",
        "(HEAP_ASYNC_GENERATOR_REQUEST_PROMISE_RECORD_OFFSET,promise_record_local)",
        "]{self.load_i64_to_local_from_offset(",
        "request_local,offset,destination_local,function);}"
    );
    let remove_head = concat!(
        "self.emit_remove_async_generator_queue_head(",
        "activation_local,request_local,next_request_local,function);"
    );
    assert_eq!(normalized_helper.matches(active_load).count(), 1);
    assert_eq!(
        normalized_helper.matches(capability_wiring).count(),
        1,
        "the active request must load its promise payload and record into their matching locals"
    );
    let active_clear = concat!(
        "self.store_i64_const_at_offset(activation_local,",
        "HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,0,function);"
    );
    assert_eq!(
        normalized_helper.matches(active_clear).count(),
        1,
        "complete-step must clear the active request after removing the queue head"
    );
    assert_eq!(normalized_helper.matches(remove_head).count(), 1);
    let active_load_position = normalized_helper
        .find(active_load)
        .expect("active-request load");
    let capability_load_position = normalized_helper
        .find(capability_wiring)
        .expect("request capability load");
    let remove_head_position = normalized_helper
        .find(remove_head)
        .expect("queue-head removal");
    let active_clear_position = normalized_helper
        .find(active_clear)
        .expect("active-request clear");
    assert!(
        active_load_position < capability_load_position
            && capability_load_position < remove_head_position
            && remove_head_position < active_clear_position,
        "complete-step must load the active request and its capability before dequeuing and clearing it"
    );
    let reject_wiring = concat!(
        "self.emit_settle_promise_record(promise_record_local,",
        "PromiseSettlement::Reject,value_payload_local,value_tag_local,function)?;"
    );
    let resolve_wiring = concat!(
        "self.emit_resolve_promise_record(promise_payload_local,promise_record_local,",
        "iterator_result_payload_local,iterator_result_tag_local,function)?;"
    );
    assert_eq!(normalized_helper.matches(reject_wiring).count(), 1);
    assert_eq!(normalized_helper.matches(resolve_wiring).count(), 1);

    let active_slots = helper
        .match_indices("HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET")
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    assert_eq!(
        active_slots.len(),
        2,
        "complete-step must load and then clear exactly one active request"
    );

    let require_active = unique_position(
        helper,
        "Instruction::I64Eqz",
        "active-request presence check",
    );
    let promise_payload = unique_position(
        helper,
        "HEAP_ASYNC_GENERATOR_REQUEST_PROMISE_PAYLOAD_OFFSET",
        "active request promise payload",
    );
    let promise_record = unique_position(
        helper,
        "HEAP_ASYNC_GENERATOR_REQUEST_PROMISE_RECORD_OFFSET",
        "active request promise record",
    );
    let remove_head = unique_position(
        helper,
        "emit_remove_async_generator_queue_head(",
        "queue-head removal",
    );
    let throw_kind = unique_position(
        helper,
        "Instruction::I64Const(COMPLETION_KIND_THROW)",
        "Throw completion branch",
    );
    let reject = unique_position(
        helper,
        "emit_settle_promise_record(",
        "Throw promise rejection",
    );
    let rejection_policy = unique_position(
        helper,
        "PromiseSettlement::Reject",
        "Throw rejection policy",
    );
    let normal_kind = unique_position(
        helper,
        "Instruction::I64Const(COMPLETION_KIND_NORMAL)",
        "Normal completion branch",
    );
    let projection = unique_position(helper, "let done = match kind {", "done projection");
    let iterator_result = unique_position(
        helper,
        "emit_iterator_result_object_from_locals(",
        "iterator-result creation",
    );
    let resolve = unique_position(
        helper,
        "emit_resolve_promise_record(",
        "Normal promise resolution",
    );
    let normalize = unique_position(
        helper,
        "set_completion_kind(CompletionKind::Normal, function)",
        "completion normalization",
    );

    assert!(
        active_slots[0] < require_active
            && require_active < promise_payload
            && require_active < promise_record
            && promise_payload < remove_head
            && promise_record < remove_head
            && remove_head < active_slots[1]
            && active_slots[1] < throw_kind
            && throw_kind < reject
            && reject < rejection_policy
            && rejection_policy < normal_kind
            && normal_kind < projection
            && projection < iterator_result
            && iterator_result < resolve
            && resolve < normalize,
        "complete-step must preserve active-request, dequeue, settlement and normalization order"
    );

    let unreachable = helper
        .match_indices("Instruction::Unreachable")
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    assert_eq!(unreachable.len(), 2);
    assert!(
        require_active < unreachable[0]
            && unreachable[0] < promise_payload
            && resolve < unreachable[1]
            && unreachable[1] < normalize,
        "missing requests and non-Normal/non-Throw completions must remain unreachable"
    );

    let reservations = local_sequence(helper, "let ", " = self.reserve_temp_local();");
    let releases = local_sequence(helper, "self.release_temp_local(", ");");
    assert_eq!(reservations.len(), 6);
    assert_eq!(helper.matches("reserve_temp_local()").count(), 6);
    assert_eq!(helper.matches("release_temp_local(").count(), 6);
    let mut expected_releases = reservations;
    expected_releases.reverse();
    assert_eq!(
        releases, expected_releases,
        "complete-step must release temporary locals in reverse reservation order"
    );
    let first_release = helper
        .find("self.release_temp_local(")
        .expect("complete-step should release its temporaries");
    assert!(normalize < first_release);
}
