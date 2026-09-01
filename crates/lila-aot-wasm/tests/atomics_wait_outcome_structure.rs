use std::fs;
use std::path::Path;

const ATOMICS_SOURCE: &str = include_str!("../src/builtins/atomics.rs");
const WAIT_ASYNC_RESULT_SOURCE: &str = include_str!("../src/builtins/atomics/wait_async_result.rs");
const DATA_SOURCE: &str = include_str!("../src/data.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

fn outcome_producer_sequence(source: &str) -> Vec<&'static str> {
    let mut occurrences = Vec::new();
    for (needle, outcome) in [
        ("AtomicsWaitOutcome::Ok", "Ok"),
        ("AtomicsWaitOutcome::NotEqual", "NotEqual"),
        ("AtomicsWaitOutcome::TimedOut", "TimedOut"),
    ] {
        occurrences.extend(
            source
                .match_indices(needle)
                .map(|(offset, _)| (offset, outcome)),
        );
    }
    occurrences.sort_unstable_by_key(|(offset, _)| *offset);
    occurrences
        .into_iter()
        .map(|(_, outcome)| outcome)
        .collect()
}

#[test]
fn wait_outcome_is_the_exact_private_non_copyable_spelling_domain() {
    assert!(ATOMICS_SOURCE.contains("\n}\n\nenum AtomicsWaitOutcome {"));
    let declaration = bounded(
        ATOMICS_SOURCE,
        "enum AtomicsWaitOutcome {",
        "\n}\n\nimpl AtomicsWaitOutcome {",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        ["Ok,", "NotEqual,", "TimedOut,"]
    );
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!ATOMICS_SOURCE.contains(&format!("{capability} for AtomicsWaitOutcome")));
    }

    let projection = normalized(bounded(
        ATOMICS_SOURCE,
        "impl AtomicsWaitOutcome {",
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]\nenum AtomicsWaitAsyncTimeoutCheckpointMode",
    ));
    assert_eq!(
        projection,
        "fnspelling(&self)->&'staticstr{matchself{Self::Ok=>\"ok\",Self::NotEqual=>\"not-equal\",Self::TimedOut=>\"timed-out\",}}}"
    );
    assert!(!projection.contains("_=>"));

    let pool_slice = normalized(bounded(
        DATA_SOURCE,
        "\"Atomics.waitAsync index out of range\",",
        "\"Atomics.xor index out of range\",",
    ));
    assert_eq!(pool_slice, "\"not-equal\",\"timed-out\",\"ok\",");
}

#[test]
fn both_result_helpers_require_and_exhaust_the_typed_outcome() {
    let wait_return = normalized(bounded(
        ATOMICS_SOURCE,
        "fn emit_atomics_wait_return_string(",
        "fn emit_atomics_wait_async(",
    ));
    assert!(wait_return.starts_with(
        "&mutself,outcome:AtomicsWaitOutcome,function:&mutFunction,){function.instruction(&Instruction::I64Const(self.strings.payload(outcome.spelling()),));"
    ));
    assert!(!wait_return.contains("value:&str"));
    assert!(!wait_return.contains("strings.payload(value)"));

    let wait_async_return = normalized(bounded(
        WAIT_ASYNC_RESULT_SOURCE,
        "pub(super) fn emit_atomics_wait_async_return_object(",
        "pub(super) fn emit_atomics_wait_async_return_promise(",
    ));
    assert!(wait_async_return.starts_with(
        "&mutself,outcome:AtomicsWaitOutcome,function:&mutFunction,)->Result<(),EmitError>{"
    ));
    assert_eq!(
        wait_async_return
            .matches("self.strings.payload(outcome.spelling())")
            .count(),
        1
    );
    assert!(!wait_async_return.contains("value:&str"));
    assert!(!wait_async_return.contains("strings.payload(value)"));

    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&src, "outcome: AtomicsWaitOutcome"),
        2
    );
    assert_eq!(count_in_rust_sources(&src, "outcome.spelling()"), 2);
}

#[test]
fn all_thirteen_semantic_producers_have_exact_variants_and_source_order() {
    let producers = ATOMICS_SOURCE
        .split_once("impl<'a> FunctionBuilder<'a> {")
        .expect("FunctionBuilder implementation")
        .1;
    assert_eq!(
        outcome_producer_sequence(producers),
        [
            "TimedOut", "Ok", "NotEqual", "TimedOut", "Ok", "Ok", "TimedOut", "TimedOut",
            "NotEqual", "TimedOut", "Ok", "NotEqual", "TimedOut",
        ]
    );
    assert_eq!(producers.matches("AtomicsWaitOutcome::Ok").count(), 4);
    assert_eq!(producers.matches("AtomicsWaitOutcome::NotEqual").count(), 3);
    assert_eq!(producers.matches("AtomicsWaitOutcome::TimedOut").count(), 6);
    for (region, expected) in [
        (
            bounded(
                ATOMICS_SOURCE,
                "fn emit_atomics_notify(",
                "fn emit_atomics_require_agent_can_suspend(",
            ),
            vec!["TimedOut", "Ok"],
        ),
        (
            bounded(
                ATOMICS_SOURCE,
                "fn emit_atomics_wait_async(",
                "pub(crate) fn emit_drain_atomics_wait_async_timeouts(",
            ),
            vec!["NotEqual", "TimedOut"],
        ),
        (
            bounded(
                ATOMICS_SOURCE,
                "fn emit_atomics_wait_async_timeout_checkpoint(",
                "fn emit_atomics_wait(",
            ),
            vec!["Ok", "Ok", "TimedOut", "TimedOut"],
        ),
        (
            ATOMICS_SOURCE
                .split_once("fn emit_atomics_wait(")
                .expect("Atomics.wait emitter")
                .1,
            vec!["NotEqual", "TimedOut", "Ok", "NotEqual", "TimedOut"],
        ),
    ] {
        assert_eq!(outcome_producer_sequence(region), expected);
    }

    let normalized_producers = normalized(producers);
    assert_eq!(
        normalized_producers
            .matches("self.strings.payload(AtomicsWaitOutcome::Ok.spelling())")
            .count(),
        3
    );
    assert_eq!(
        normalized_producers
            .matches("self.strings.payload(AtomicsWaitOutcome::TimedOut.spelling())")
            .count(),
        3
    );
    assert!(!normalized_producers
        .contains("self.strings.payload(AtomicsWaitOutcome::NotEqual.spelling())"));
    for (call, expected) in [
        (
            "self.emit_atomics_wait_async_return_object(AtomicsWaitOutcome::NotEqual,function)?;",
            1,
        ),
        (
            "self.emit_atomics_wait_async_return_object(AtomicsWaitOutcome::TimedOut,function)?;",
            1,
        ),
        (
            "self.emit_atomics_wait_return_string(AtomicsWaitOutcome::Ok,function);",
            1,
        ),
        (
            "self.emit_atomics_wait_return_string(AtomicsWaitOutcome::NotEqual,function);",
            2,
        ),
        (
            "self.emit_atomics_wait_return_string(AtomicsWaitOutcome::TimedOut,function);",
            2,
        ),
    ] {
        assert_eq!(
            normalized_producers.matches(call).count(),
            expected,
            "{call}"
        );
    }

    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(count_in_rust_sources(&src, "AtomicsWaitOutcome::"), 13);
    assert_eq!(count_in_rust_sources(&src, "AtomicsWaitOutcome"), 17);
    for literal in ["\"ok\"", "\"not-equal\"", "\"timed-out\""] {
        assert_eq!(
            count_in_rust_sources(&src, literal),
            2,
            "only the spelling projection and pinned string pool may contain `{literal}`"
        );
    }
}

#[test]
fn wasm_and_host_wait_statuses_remain_numeric_before_outcome_projection() {
    let atomics = normalized(ATOMICS_SOURCE);
    assert!(atomics.contains(
        "AgentHostOperation::NotifyAsyncWaiters.wire(),));function.instruction(&Instruction::LocalGet(address_local));function.instruction(&Instruction::LocalGet(count_local));function.instruction(&Instruction::Call(agent_call_function_index));function.instruction(&Instruction::LocalSet(claimed_local));"
    ));
    let notify = normalized(bounded(
        ATOMICS_SOURCE,
        "fn emit_atomics_notify(",
        "fn emit_atomics_require_agent_can_suspend(",
    ));
    let notify_poll = bounded(
        &notify,
        "AgentHostOperation::PollAsyncWaiter.wire(),));",
        "self.emit_settle_promise_record(",
    );
    assert!(notify_poll.contains(
        "function.instruction(&Instruction::LocalGet(waiter_host_id_local));function.instruction(&Instruction::I64Const(0));function.instruction(&Instruction::Call(agent_call_function_index));function.instruction(&Instruction::I64Const(1));function.instruction(&Instruction::I64Eq);"
    ));
    assert_eq!(
        notify_poll
            .matches("AtomicsWaitOutcome::Ok.spelling()")
            .count(),
        1
    );
    assert_eq!(
        notify_poll
            .matches("function.instruction(&Instruction::If(BlockType::Empty));")
            .count(),
        1
    );
    let notify_status_decode = notify_poll
        .find("function.instruction(&Instruction::I64Eq);")
        .expect("notify poll success status decode");
    let notify_success_branch = notify_poll
        .find("function.instruction(&Instruction::If(BlockType::Empty));")
        .expect("notify poll success branch");
    let notify_ok_outcome = notify_poll
        .find("AtomicsWaitOutcome::Ok.spelling()")
        .expect("notify poll ok outcome");
    assert!(notify_status_decode < notify_success_branch);
    assert!(notify_success_branch < notify_ok_outcome);
    assert!(!notify_poll.contains("AtomicsWaitOutcome::NotEqual"));
    assert!(!notify_poll.contains("AtomicsWaitOutcome::TimedOut"));

    let wait_async_result = normalized(WAIT_ASYNC_RESULT_SOURCE);
    assert!(wait_async_result.contains(
        "AgentHostOperation::RegisterAsyncWaiter.wire(),));function.instruction(&Instruction::LocalGet(address_local));function.instruction(&Instruction::I64Const(0));function.instruction(&Instruction::Call(agent_call_function_index));function.instruction(&Instruction::LocalSet(waiter_host_id_local));"
    ));

    let wait = normalized(
        ATOMICS_SOURCE
            .split_once("fn emit_atomics_wait(")
            .expect("Atomics.wait emitter")
            .1,
    );
    assert_eq!(wait.matches("Instruction::MemoryAtomicWait64(").count(), 1);
    assert_eq!(wait.matches("Instruction::MemoryAtomicWait32(").count(), 1);
    assert!(wait.contains(
        "Instruction::LocalGet(wait_result_local));function.instruction(&Instruction::I64Eqz);function.instruction(&Instruction::If(BlockType::Empty));self.emit_atomics_wait_return_string(AtomicsWaitOutcome::Ok,function);"
    ));
    assert!(wait.contains(
        "Instruction::LocalGet(wait_result_local));function.instruction(&Instruction::I64Const(1));function.instruction(&Instruction::I64Eq);function.instruction(&Instruction::If(BlockType::Empty));self.emit_atomics_wait_return_string(AtomicsWaitOutcome::NotEqual,function);"
    ));

    let checkpoint = normalized(bounded(
        ATOMICS_SOURCE,
        "fn emit_atomics_wait_async_timeout_checkpoint(",
        "fn emit_atomics_wait(",
    ));
    let checkpoint_poll = bounded(
        &checkpoint,
        "AgentHostOperation::PollAsyncWaiter.wire(),));",
        "AgentHostOperation::CancelAsyncWaiter.wire(),));",
    );
    assert!(checkpoint_poll.contains(
        "function.instruction(&Instruction::LocalGet(waiter_host_id_local));function.instruction(&Instruction::I64Const(0));function.instruction(&Instruction::Call(agent_call_function_index));function.instruction(&Instruction::LocalSet(host_waiter_status_local));function.instruction(&Instruction::LocalGet(host_waiter_status_local));function.instruction(&Instruction::I64Eqz);function.instruction(&Instruction::If(BlockType::Empty));function.instruction(&Instruction::Else);"
    ));
    assert!(checkpoint_poll.contains(
        "function.instruction(&Instruction::LocalGet(host_waiter_status_local));function.instruction(&Instruction::I64Const(1));function.instruction(&Instruction::I64Eq);function.instruction(&Instruction::If(BlockType::Empty));self.load_i64_to_local_from_offset(waiter_local,HEAP_ATOMICS_ASYNC_WAITER_PROMISE_RECORD_OFFSET,promise_record_local,function,);function.instruction(&Instruction::I64Const(self.strings.payload(AtomicsWaitOutcome::Ok.spelling()),));"
    ));
    assert!(!checkpoint_poll.contains("AtomicsWaitOutcome::NotEqual"));
    assert!(!checkpoint_poll.contains("AtomicsWaitOutcome::TimedOut"));

    let checkpoint_cancel = checkpoint
        .split_once("AgentHostOperation::CancelAsyncWaiter.wire(),));")
        .expect("checkpoint CancelAsyncWaiter decode")
        .1;
    assert!(checkpoint_cancel.contains(
        "function.instruction(&Instruction::LocalGet(waiter_host_id_local));function.instruction(&Instruction::I64Const(0));function.instruction(&Instruction::Call(agent_call_function_index));function.instruction(&Instruction::LocalSet(host_waiter_status_local));"
    ));
    assert!(checkpoint_cancel.contains(
        "function.instruction(&Instruction::LocalGet(host_waiter_status_local));function.instruction(&Instruction::I64Const(1));function.instruction(&Instruction::I64Eq);function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));function.instruction(&Instruction::I64Const(self.strings.payload(AtomicsWaitOutcome::Ok.spelling()),));function.instruction(&Instruction::Else);function.instruction(&Instruction::I64Const(self.strings.payload(AtomicsWaitOutcome::TimedOut.spelling()),));function.instruction(&Instruction::End);"
    ));
    assert!(!checkpoint_cancel.contains("AtomicsWaitOutcome::NotEqual"));
}
