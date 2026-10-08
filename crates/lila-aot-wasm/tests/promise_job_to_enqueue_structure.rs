const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");

const PROMISE_JOB_TO_ENQUEUE_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_job_to_enqueue.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

#[test]
fn promise_job_to_enqueue_is_one_private_non_copy_payload_authority() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_job_to_enqueue;\n")
            .count(),
        1
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_job_to_enqueue;"));
    assert!(!PROMISE_SOURCE.contains("promise_job_to_enqueue::"));
    assert!(!PROMISE_SOURCE.contains("PromiseJobToEnqueue"));
    assert!(!PROMISE_JOB_TO_ENQUEUE_SOURCE.contains("pub enum PromiseJobToEnqueue"));
    let declaration = bounded(
        PROMISE_JOB_TO_ENQUEUE_SOURCE,
        "use super::*;",
        "impl FunctionBuilder<'_>",
    );
    assert!(!declaration.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!PROMISE_JOB_TO_ENQUEUE_SOURCE
            .contains(&format!("impl {capability} for PromiseJobToEnqueue")));
    }
}
