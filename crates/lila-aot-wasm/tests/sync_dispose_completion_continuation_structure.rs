const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/sync-dispose-completion-continuation.md");
const TASK: &str = include_str!("../../../tasks/15-generators-iterators-resource-management.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[test]
fn continuation_is_must_use_and_capability_free() {
    let declaration = bounded(
        CONTROL_FLOW_SOURCE,
        "#[must_use = \"a sync disposal continuation must be consumed after completion restoration\"]",
        "fn innermost_target(",
    );
    assert!(!declaration.contains("#[derive("));
    assert_eq!(
        without_whitespace(bounded(
            declaration,
            "enum SyncDisposeCompletionContinuation {",
            "}",
        )),
        "Dispatch,DispatchAsyncFunction,DispatchAsyncGenerator,DeferToIteratorClose,"
    );

    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
    ] {
        assert!(!CONTROL_FLOW_SOURCE.contains(&format!(
            "impl {capability} for SyncDisposeCompletionContinuation"
        )));
    }
}

#[test]
fn producers_move_one_continuation_into_the_sole_exhaustive_consumer() {
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("fn completion_continuation(&self) -> SyncDisposeCompletionContinuation")
            .count(),
        1
    );
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("owner.completion_continuation()")
            .count(),
        1
    );
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("SyncDisposeCompletionContinuation::Dispatch,")
            .count(),
        3
    );
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("SyncDisposeCompletionContinuation::DispatchAsyncFunction,")
            .count(),
        1
    );
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("SyncDisposeCompletionContinuation::DispatchAsyncGenerator,")
            .count(),
        1
    );
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("SyncDisposeCompletionContinuation::DeferToIteratorClose,")
            .count(),
        1
    );

    let consumer = bounded(
        CONTROL_FLOW_SOURCE,
        "fn consume_sync_disposable_resources(",
        "pub(crate) fn compile_try_catch_finally(",
    );
    assert!(consumer.contains("continuation: SyncDisposeCompletionContinuation"));
    assert_eq!(consumer.matches("match continuation {").count(), 1);
    for variant in [
        "Dispatch",
        "DispatchAsyncFunction",
        "DispatchAsyncGenerator",
        "DeferToIteratorClose",
    ] {
        assert_eq!(
            consumer
                .matches(&format!("SyncDisposeCompletionContinuation::{variant} =>"))
                .count(),
            1
        );
    }
    assert!(!consumer.contains("_ =>"));
    assert!(!consumer.contains("continuation.clone()"));
    let restore = consumer
        .find("self.restore_saved_completion(")
        .expect("saved completion restoration");
    let dispatch = consumer
        .find("match continuation {")
        .expect("continuation dispatch");
    assert!(restore < dispatch);
}

#[test]
fn contract_and_task_record_the_one_way_continuation_boundary() {
    for evidence in [CONTRACT, TASK] {
        let evidence = without_whitespace(evidence);
        assert!(evidence.contains("must-use"));
        assert!(evidence.contains("capability-free"));
        assert!(evidence.contains("soleownership-consumingcontinuationmatch"));
        assert!(evidence.contains("completionrestoration"));
        assert!(evidence.contains("noemittedWasmorruntimebehavior"));
        assert!(evidence.contains("BatchAC"));
    }
}
