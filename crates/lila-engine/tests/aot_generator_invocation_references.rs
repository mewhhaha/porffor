use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_trace(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compilation worker");
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
        .expect("generator invocation compiles and executes through Wasm AOT");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}\nsource:\n{source}",
        observed.completion
    );
    assert_eq!(
        observed.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).into()))
            .collect::<Vec<_>>(),
        "source:\n{source}"
    );
}

#[test]
fn spread_results_and_callee_remain_private_activation_snapshots() {
    assert_trace(
        include_str!("fixtures/generator_invocation_references/spread_snapshot.js"),
        &[
            "iterator",
            "next-get",
            "next:0",
            "value:0",
            "next:1",
            "value:1",
            "next:2",
            "argument?",
            "spread?",
            "tail-iterator",
            "tail-next:0",
            "tail-next:1",
            "call:20:1:2:3:4",
            "30:true",
        ],
    );
}

#[test]
fn abrupt_spread_skips_close_and_later_operands_while_noncallability_waits_for_arguments() {
    assert_trace(
        include_str!("fixtures/generator_invocation_references/spread_abrupt.js"),
        &[
            "get",
            "iterator",
            "next",
            "value",
            "true",
            "get",
            "pending",
            "finite-iterator",
            "after",
            "true",
        ],
    );
}

#[test]
fn constructor_identity_is_saved_but_its_prototype_is_read_after_arguments() {
    assert_trace(
        include_str!("fixtures/generator_invocation_references/construct_reference.js"),
        &[
            "constructor?",
            "get",
            "before",
            "argument?",
            "original:42:true",
            "42:true",
            "true",
            "bad?",
            "after",
            "true",
        ],
    );
}

#[test]
fn tag_reference_and_template_identity_survive_yielded_substitutions() {
    assert_trace(
        include_str!("fixtures/generator_invocation_references/tag_reference.js"),
        &[
            "key?",
            "get",
            "substitution?",
            "after",
            "tag:true:20:3:9",
            "head:tail:end",
            "true:true:false",
            "3:true",
            "key?",
            "get",
            "substitution?",
            "after",
            "tag:true:20:4:9",
            "head:tail:end",
            "true:true:true",
            "4:true",
        ],
    );
}

#[test]
fn private_and_super_references_keep_receiver_get_timing_and_brand_failure() {
    assert_trace(
        include_str!("fixtures/generator_invocation_references/private_super_reference.js"),
        &[
            "private-get",
            "private?",
            "private:true:1",
            "super-key?",
            "key",
            "super-get",
            "super-argument?",
            "super:true:2",
            "private-base?",
            "private-get",
            "private-argument?",
            "private:true:3",
            "3:true",
            "brand?",
            "true",
        ],
    );
}

#[test]
fn private_and_super_getter_effects_precede_even_nonsuspending_arguments() {
    assert_trace(
        include_str!("fixtures/generator_invocation_references/reference_get_effects.js"),
        &[
            "private-get",
            "private-argument",
            "private?",
            "private-call:10:2",
            "super-get",
            "super-argument",
            "super?",
            "super-call:20:3",
            "true",
        ],
    );
}

#[test]
fn with_reference_does_not_repeat_binding_selection_after_resume() {
    assert_trace(
        include_str!("fixtures/generator_invocation_references/with_reference.js"),
        &[
            "has",
            "unscopables",
            "blocked",
            "has",
            "get",
            "with?",
            "call:true:7",
            "true",
        ],
    );
}

#[test]
fn runtime_environment_reference_retains_the_resolved_with_call_base() {
    assert_trace(
        include_str!("fixtures/generator_invocation_references/runtime_environment_reference.js"),
        &[
            "has",
            "unscopables",
            "blocked",
            "has",
            "get",
            "environment?",
            "call:true:7",
            "true",
        ],
    );
}

#[test]
fn direct_eval_uses_the_original_identity_and_shadowed_eval_remains_an_ordinary_call() {
    assert_trace(
        include_str!("fixtures/generator_invocation_references/direct_eval_reference.js"),
        &["eval?", "42:true", "custom?", "custom:true:7", "7:true"],
    );
}

#[test]
fn grouped_optional_callees_keep_conditional_receivers_and_outer_argument_order() {
    assert_trace(
        include_str!("fixtures/generator_invocation_references/optional_reference.js"),
        &[
            "get",
            "optional?",
            "call:true:20:7",
            "7:true",
            "nullish?",
            "after",
            "true",
            "factory",
            "plain?",
            "plain:true:8",
            "8:true",
            "tag-get",
            "tag?",
            "tag:true:9",
            "9:true",
        ],
    );
}

#[test]
fn computed_key_effects_invalidate_base_facts_without_replacing_saved_identities() {
    assert_trace(
        include_str!("fixtures/generator_invocation_references/computed_key_effects.js"),
        &[
            "key-source",
            "argument?",
            "new:true:20:7",
            "7:true",
            "left",
            "right",
            "key-source",
            "resumed",
            "key-source",
            "resumed",
            "new:true:20:2",
            "2:true",
            "new:true:20:1",
            "1:true",
        ],
    );
}

#[test]
fn later_argument_effects_keep_saved_object_identity_and_discard_stale_content_facts() {
    assert_trace(
        include_str!("fixtures/generator_invocation_references/argument_object_effects.js"),
        &[
            "argument?",
            "resumed:true:7",
            "7:true",
            "mutate",
            "eager?",
            "mutated:true:10",
            "10:true",
        ],
    );
}

#[test]
fn abrupt_resume_skips_pending_invocations_spreads_and_substitutions() {
    assert_trace(
        include_str!("fixtures/generator_invocation_references/abrupt_resume.js"),
        &[
            "first",
            "pending",
            "true",
            "first",
            "pending",
            "23:true",
            "construct?",
            "true",
            "tag?",
            "24:true",
        ],
    );
}

#[test]
fn nested_direct_yield_operands_stage_their_invocations_before_the_outer_yield() {
    assert_trace(
        include_str!("fixtures/generator_invocation_references/nested_yield_operand.js"),
        &[
            "inner?",
            "construct:7",
            "7:false",
            "9:true",
            "substitution?",
            "tag:3",
            "tag3:false",
            "12:true",
        ],
    );
}
