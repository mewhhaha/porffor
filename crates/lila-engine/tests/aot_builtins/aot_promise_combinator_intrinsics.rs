use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_combinator_results(source: &str, expected_results: usize) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
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
        .expect("Promise combinators use initialized realm intrinsics through Wasm");
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    assert_eq!(
        outcome.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string()); expected_results],
        "source:\n{source}",
    );
}

#[test]
fn all_and_all_settled_materialize_nonempty_elements_after_bootstrap() {
    assert_combinator_results(
        r#"
var IntrinsicPromise = Promise;
var marker = {};
globalThis.AggregateError = function() { throw 'mutable AggregateError was read'; };
globalThis.Promise = function() { throw 'mutable Promise was read'; };
IntrinsicPromise.all([]).then(function(values) { print(values.length === 0); });
IntrinsicPromise.all([IntrinsicPromise.resolve(11), 23]).then(function(values) {
  print(values.length === 2 && values[0] === 11 && values[1] === 23);
});
IntrinsicPromise.allSettled([IntrinsicPromise.resolve(11), IntrinsicPromise.reject(marker)])
  .then(function(results) {
    print(results.length === 2 && results[0].status === 'fulfilled' && results[0].value === 11
      && results[1].status === 'rejected' && results[1].reason === marker);
  });
void 0;
"#,
        3,
    );
}

#[test]
fn any_uses_the_canonical_aggregate_error_for_empty_and_rejected_inputs() {
    assert_combinator_results(
        r#"
var IntrinsicPromise = Promise;
var IntrinsicAggregateError = AggregateError;
var marker = {};
globalThis.AggregateError = function() { throw 'mutable AggregateError was read'; };
IntrinsicPromise.any([]).then(undefined, function(error) {
  print(Object.getPrototypeOf(error) === IntrinsicAggregateError.prototype && error.errors.length === 0);
});
IntrinsicPromise.any([IntrinsicPromise.reject(marker), IntrinsicPromise.reject(23)])
  .then(undefined, function(error) {
    print(Object.getPrototypeOf(error) === IntrinsicAggregateError.prototype
      && error.errors.length === 2 && error.errors[0] === marker && error.errors[1] === 23);
  });
void 0;
"#,
        2,
    );
}

#[test]
fn borrowed_foreign_combinators_separate_method_and_capability_realms() {
    assert_combinator_results(
        r#"
var other = __lilaCreateRealm().global;
var EntryPromise = Promise;
var ForeignPromise = other.Promise;
var ForeignArray = other.Array;
var ForeignAggregateError = other.AggregateError;
var entryAggregateError = AggregateError;
other.AggregateError = function() { throw 'foreign AggregateError was read'; };
other.Promise = function() { throw 'foreign Promise was read'; };
var all = ForeignPromise.all.call(EntryPromise, [11, 23]);
all.then(function(values) {
  print(all instanceof EntryPromise && !(all instanceof ForeignPromise)
    && Object.getPrototypeOf(values) === ForeignArray.prototype && values.join(',') === '11,23');
});
var empty = ForeignPromise.any.call(EntryPromise, []);
empty.then(undefined, function(error) {
  print(empty instanceof EntryPromise && error instanceof ForeignAggregateError
    && !(error instanceof entryAggregateError)
    && Object.getPrototypeOf(error.errors) === ForeignArray.prototype && error.errors.length === 0);
});
var rejected = ForeignPromise.any.call(EntryPromise, [EntryPromise.reject(17)]);
rejected.then(undefined, function(error) {
  print(rejected instanceof EntryPromise && error instanceof ForeignAggregateError
    && !(error instanceof entryAggregateError)
    && Object.getPrototypeOf(error.errors) === ForeignArray.prototype && error.errors[0] === 17);
});
void 0;
"#,
        3,
    );
}
