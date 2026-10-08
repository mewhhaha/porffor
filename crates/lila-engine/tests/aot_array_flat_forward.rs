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
        .expect("forward Flat traversal compiles and executes through Wasm AOT");
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
fn root_length_precedes_depth_and_species_and_indices_remain_live() {
    assert_trace(
        include_str!("fixtures/array_flat_forward/root_order_and_live_indices.js"),
        &[
            "root-get:length",
            "length-number",
            "depth-number",
            "root-get:constructor",
            "constructor",
            "species",
            "create:0",
            "root-has:0",
            "root-get:0",
            "zero",
            "define:0:3",
            "root-has:1",
            "root-get:1",
            "define:1:9",
            "3:9:false",
            "live-has:0",
            "live-get:0",
            "mutate",
            "live-has:1",
            "live-has:2",
            "live-get:2",
            "live-has:3",
            "live-get:3",
            "3:1:3:4",
        ],
    );
}

#[test]
fn depth_uses_full_number_coercion_and_preserves_infinite_and_zero_cases() {
    assert_trace(
        include_str!("fixtures/array_flat_forward/depth_coercions.js"),
        &[
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "true",
            "7",
            "7",
            "valueOf",
            "toString",
            "true",
            "number",
            "true",
            "true",
            "true",
            "true:true:true",
        ],
    );
}

#[test]
fn nested_sources_snapshot_length_on_entry_then_resume_parent_in_order() {
    assert_trace(
        include_str!("fixtures/array_flat_forward/nested_frames_and_proxy_order.js"),
        &[
            "root-get:length",
            "root-has:0",
            "root-get:0",
            "child-get:length",
            "child-has:0",
            "child-get:0",
            "grand-get:length",
            "grand-has:0",
            "grand-get:0",
            "grand-zero",
            "grand-has:1",
            "grand-get:1",
            "child-has:1",
            "child-get:1",
            "root-has:1",
            "root-get:1",
            "4:4:5:20:30",
        ],
    );
}

#[test]
fn holes_use_inherited_properties_and_only_arrays_flatten_without_iteration() {
    assert_trace(
        include_str!("fixtures/array_flat_forward/holes_inheritance_and_nonarrays.js"),
        &["root-inherited", "child-inherited", "6:1:2:3:true:true:4"],
    );
}

#[test]
fn abrupt_completions_keep_identity_and_depth_zero_does_not_inspect_child_brand() {
    assert_trace(
        include_str!("fixtures/array_flat_forward/abrupt_identity_and_zero_depth.js"),
        &[
            "length-get",
            "length:true",
            "depth-length",
            "depth-number",
            "depth:true",
            "constructor",
            "species:true",
            "has:0",
            "has:true",
            "get-zero",
            "get:true",
            "nested-length",
            "nested:true",
            "true",
            "true",
        ],
    );
}

#[test]
fn target_definition_checks_array_descriptors_and_keeps_proxy_abrupt_identity() {
    assert_trace(
        include_str!("fixtures/array_flat_forward/target_definition_and_abrupt.js"),
        &[
            "define:0:7",
            "true",
            "refuse",
            "true",
            "true",
            "true",
            "7:true:true:true:false",
        ],
    );
}

#[test]
fn huge_lengths_and_depths_reach_property_operations_without_integer_traps() {
    assert_trace(
        include_str!("fixtures/array_flat_forward/huge_lengths_and_nested_length_coercion.js"),
        &[
            "get:length",
            "has:0",
            "true",
            "get:length",
            "has:0",
            "true",
            "get:length",
            "has:0",
            "true",
            "nested-get",
            "nested-number",
            "nested-has:0",
            "true",
            "0",
            "0",
            "0",
            "0",
            "0",
        ],
    );
}

#[test]
fn nested_flat_invocations_own_independent_ancestor_frames() {
    assert_trace(
        include_str!("fixtures/array_flat_forward/reentrant_activation_frames.js"),
        &["enter", "inner:7", "4:1:2:3:4"],
    );
}

#[test]
fn generic_receivers_use_boxing_and_observable_length_owners() {
    assert_trace(
        include_str!("fixtures/array_flat_forward/generic_receivers.js"),
        &[
            "2:a:b",
            "0",
            "1:11",
            "typed-length",
            "1:10",
            "arguments-length",
            "1:13",
        ],
    );
}
