use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_trace(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compiler worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("JSON.parse must execute its canonical emitted parser and reviver walk");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            expected
                .iter()
                .map(|line| HostOutputEvent::PrintLine((*line).into()))
                .collect::<Vec<_>>(),
            "{source}"
        );
    }
}

#[test]
fn same_tag_sibling_replacement_enters_actual_new_descendants() {
    assert_trace(
        include_str!("fixtures/json_canonical_reviver/same_tag_replacement.js"),
        &[
            "start:0",
            "inserted:none",
            "0:none",
            "1:none",
            "2:none",
            "array:none",
            "deep:none",
            "fresh:none",
            "last:none",
            "object:none",
            "<root>:none",
            "true:true:3:7",
        ],
    );
}

#[test]
fn inserted_functions_and_arguments_supply_real_callback_holders() {
    assert_trace(
        include_str!("fixtures/json_canonical_reviver/function_arguments_descendants.js"),
        &[
            "first:0",
            "leaf:none",
            "branch:none",
            "tail:none",
            "function-effect",
            "function-call:called:1",
            "function:none",
            "leaf:none",
            "0:none",
            "0:none",
            "extra:none",
            "arguments:none",
            "<root>:none",
            "true:13:15:6:1",
        ],
    );
}

#[test]
fn collection_entry_snapshots_preserve_live_forward_reads_and_inherited_indexes() {
    assert_trace(
        include_str!("fixtures/json_canonical_reviver/live_snapshots.js"),
        &[
            "call:kick:number",
            "call:0:number",
            "inherited-one",
            "call:1:number",
            "call:2:number",
            "call:item:object",
            "get-a",
            "call:a:number",
            "call:b:undefined",
            "call:box:object",
            "call:<root>:object",
            "4:7:30:99:false:4",
        ],
    );
}

#[test]
fn source_context_uses_same_value_and_final_duplicate_source_without_static_values() {
    assert_trace(
        include_str!("fixtures/json_canonical_reviver/source_context_mutation.js"),
        &[
            "change:0",
            "zero:none",
            "same:1e+0",
            "different:none",
            "n:none",
            "nest:none",
            "dup:5",
            "own:6",
            "__proto__:none",
            "<empty>:7",
            "<root>:none",
            "true:false:9:false:true:true",
        ],
    );
}

#[test]
fn ordinary_calls_acquire_callee_once_then_evaluate_all_operands_before_coercion() {
    assert_trace(
        include_str!("fixtures/json_canonical_reviver/ordinary_operand_order.js"),
        &[
            "receiver",
            "get-parse",
            "input",
            "reviver",
            "ignored",
            "coerce:string",
            "call:value:1:1",
            "call:<root>:2:none",
            "1:1:2",
            "callee",
            "1:2",
        ],
    );
}

#[test]
fn canonical_mutation_and_callback_failures_preserve_arbitrary_throw_identity() {
    assert_trace(
        include_str!("fixtures/json_canonical_reviver/abrupt_mutation.js"),
        &[
            "visit:first",
            "length",
            "true",
            "own-visit:first",
            "ownKeys",
            "true",
            "callback:b",
            "true:1",
            "input-throw",
            "true",
            "7",
        ],
    );
}

#[test]
fn private_parse_records_do_not_observe_inherited_hooks_for_new_current_keys() {
    assert_trace(
        include_str!("fixtures/json_canonical_reviver/private_metadata_names.js"),
        &[
            "first:0",
            "keep:1",
            "current-get:z",
            "z:none",
            "object:none",
            "<root>:none",
            "10:true:0",
        ],
    );
}

// The GC cohort is independent of the preserved canonical trace cohorts above.
fn assert_json_gc_source(source: &str, marker: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    host_surface_policy: lila_engine::HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("the finite JSON GC cohort uses the compiled Wasm backend");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(lila_engine::ObservedJsValue::Number(
                lila_engine::ObservedNumber::from_f64(262.0)
            ))
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(marker.into())]
        );
    }
}

#[test]
fn gc_parser_reviver_retains_utf16_source_roots_mutations_and_error_realms() {
    assert_json_gc_source(
        include_str!("fixtures/json_canonical_reviver/gc_utf16_and_realms.js"),
        "json-gc-parse-reviver:ok",
    );
}
