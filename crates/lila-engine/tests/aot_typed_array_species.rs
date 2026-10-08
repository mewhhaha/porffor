use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_species(source: &str, output: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray species controls execute through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(output.into())],
            "{source}"
        );
    }
}

#[test]
fn all_element_kinds_keep_defining_realm_defaults_and_proxy_constructors() {
    assert_species(
        include_str!("fixtures/typed_array_species/constructors_and_realms.js"),
        "typed-array-species-constructors:ok",
    );
}

#[test]
fn method_specific_species_order_and_abrupt_values_are_observable() {
    assert_species(
        include_str!("fixtures/typed_array_species/ordering_and_abrupt.js"),
        "typed-array-species-order:ok",
    );
}

#[test]
fn result_brand_content_length_and_live_views_keep_their_distinct_rules() {
    assert_species(
        include_str!("fixtures/typed_array_species/views_and_results.js"),
        "typed-array-species-views:ok",
    );
}

#[test]
fn sparse_source_borrows_entry_methods_for_a_created_realm_element_kind() {
    // The host Realm surface already requests full standard globals. This is
    // semantic borrowed-default evidence, not an isolated installer-plan proof.
    assert_species(
        r#"
var foreign = __lilaCreateRealm().global;
var methods = [Uint8Array.prototype.map, Uint8Array.prototype.filter,
  Uint8Array.prototype.slice, Uint8Array.prototype.subarray];
for (var index = 0; index < methods.length; index++) {
  var source = new foreign.Float16Array([1.5, 2.5]);
  source.constructor = undefined;
  var result = index === 0 ? methods[index].call(source, function(value) { return value; })
    : index === 1 ? methods[index].call(source, function() { return true; })
    : methods[index].call(source, 0, 2);
  if (result.constructor.name !== 'Float16Array' || result.length !== 2 ||
      result[0] !== 1.5 || result[1] !== 2.5 ||
      Object.getPrototypeOf(result) === foreign.Float16Array.prototype) throw 'sparse borrowed default';
}
print('typed-array-species-sparse:ok');
262;
"#,
        "typed-array-species-sparse:ok",
    );
}
