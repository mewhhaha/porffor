use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_trace(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("compiled suspended invocation");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(matches!(observed.completion, ObservedCompletion::Normal(_)));
    assert_eq!(
        observed.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).into()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn member_key_and_replacement() {
    assert_trace(
        include_str!("../fixtures/suspended_call_references/member_key_and_replacement.js"),
        &[
            "base",
            "key-source",
            "key-convert",
            "get",
            "first",
            "operand",
            "called",
            "replace",
            "call:true:1:2",
            "done",
        ],
    );
}

#[test]
fn getter_and_key_abrupt() {
    assert_trace(
        include_str!("../fixtures/suspended_call_references/getter_and_key_abrupt.js"),
        &[
            "get",
            "getter:true",
            "key",
            "key:true",
            "proxy-get",
            "proxy:true",
            "done",
            "called",
        ],
    );
}

#[test]
fn noncallable_and_nonconstructor_after_arguments() {
    assert_trace(
        include_str!(
            "../fixtures/suspended_call_references/noncallable_and_nonconstructor_after_arguments.js"
        ),
        &[
            "get",
            "argument:call",
            "called",
            "call:true",
            "get",
            "argument:new",
            "new:true",
            "done",
        ],
    );
}

#[test]
fn spread_evaluation_and_private_list() {
    assert_trace(
        include_str!("../fixtures/suspended_call_references/spread_evaluation_and_private_list.js"),
        &[
            "lead",
            "iterator-get",
            "iterator-call",
            "next-get",
            "next:0",
            "done:0",
            "value:0",
            "next:1",
            "done:1",
            "value:1",
            "next:2",
            "done:2",
            "operand",
            "called",
            "replace",
            "tail",
            "values:5:10:20:99:30",
            "done",
        ],
    );
}

#[test]
fn spread_abrupt_does_not_close_or_evaluate_later() {
    assert_trace(
        include_str!(
            "../fixtures/suspended_call_references/spread_abrupt_does_not_close_or_evaluate_later.js"
        ),
        &[
            "callee",
            "iterator",
            "next",
            "done-get",
            "value-get",
            "caught:true",
            "done",
            "called",
        ],
    );
}

#[test]
fn constructor_capture() {
    assert_trace(
        include_str!("../fixtures/suspended_call_references/constructor_capture.js"),
        &[
            "get-constructor",
            "operand",
            "called",
            "replace",
            "construct:7:Original",
            "result:7:true",
        ],
    );
}

#[test]
fn tag_reference_capture() {
    assert_trace(
        include_str!("../fixtures/suspended_call_references/tag_reference_capture.js"),
        &[
            "tag-get",
            "operand",
            "called",
            "replace",
            "tag:true:true:true:head:tail:7",
            "done",
        ],
    );
}

#[test]
fn direct_eval_original_reference() {
    assert_trace(
        include_str!("../fixtures/suspended_call_references/direct_eval_original_reference.js"),
        &["operand", "called", "replace", "eval:42"],
    );
}

#[test]
fn private_and_super_members() {
    assert_trace(
        include_str!("../fixtures/suspended_call_references/private_and_super_members.js"),
        &[
            "private-get",
            "operand",
            "called",
            "private:true:7",
            "super-get",
            "operand",
            "super:true:7",
            "done",
        ],
    );
}

#[test]
fn with_reference() {
    assert_trace(
        include_str!("../fixtures/suspended_call_references/with_reference.js"),
        &[
            "has",
            "unscopables",
            "blocked",
            "has",
            "get",
            "operand",
            "called",
            "replace",
            "call:true:7",
            "done",
        ],
    );
}

#[test]
fn runtime_environment_reference() {
    assert_trace(
        include_str!("../fixtures/suspended_call_references/runtime_environment_reference.js"),
        &[
            "has",
            "unscopables",
            "blocked",
            "has",
            "get",
            "operand",
            "called",
            "replace",
            "call:true:7",
            "done",
        ],
    );
}

#[test]
fn optional_outer_call_admission() {
    assert_trace(
        include_str!("../fixtures/suspended_call_references/optional_outer_call_admission.js"),
        &["called", "7"],
    );
}

#[test]
fn optional_reference_replacement() {
    assert_trace(
        include_str!("../fixtures/suspended_call_references/optional_reference_replacement.js"),
        &[
            "base",
            "key",
            "key-convert",
            "get",
            "operand",
            "called",
            "replace",
            "call:true:7",
            "done",
        ],
    );
}

#[test]
fn optional_nullish_outer_arguments() {
    assert_trace(
        include_str!("../fixtures/suspended_call_references/optional_nullish_outer_arguments.js"),
        &[
            "base",
            "argument:nullish",
            "called",
            "nullish:true",
            "get",
            "argument:noncallable",
            "noncallable:true",
            "done",
        ],
    );
}

#[test]
fn optional_chain_call_resets_receiver() {
    assert_trace(
        include_str!(
            "../fixtures/suspended_call_references/optional_chain_call_resets_receiver.js"
        ),
        &["inner:true", "operand", "called", "outer:true:7", "done"],
    );
}

#[test]
fn optional_tag_reference() {
    assert_trace(
        include_str!("../fixtures/suspended_call_references/optional_tag_reference.js"),
        &[
            "get",
            "operand",
            "called",
            "replace",
            "tag:true:head:7",
            "nullish-operand",
            "nullish:true",
            "done",
        ],
    );
}

#[test]
fn optional_reference_abrupt() {
    assert_trace(
        include_str!("../fixtures/suspended_call_references/optional_reference_abrupt.js"),
        &[
            "get",
            "getter:true",
            "key-convert",
            "key:true",
            "done",
            "called",
        ],
    );
}
