use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_stack(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    let source = format!("function assert(v,m) {{ if (!v) throw new Error(m); }}\n{source}");
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
        .expect("DisposableStack control executes through Wasm AOT");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observed.completion
    );
}

#[test]
fn use_keeps_original_list_when_dispose_method_getter_moves_the_stack() {
    assert_stack(
        r#"
var source = new DisposableStack(), moved, order = [], original = {};
source.defer(function() {order.push('old');});
Object.defineProperty(original, Symbol.dispose, {get: function() {
    moved = source.move();
    return function() {'use strict'; assert(this === original, 'whole use receiver'); order.push('new');};
}});
assert(source.use(original) === original && source.disposed && !moved.disposed, 'pending check precedes getter move');
moved.dispose();
assert(order.join(',') === 'new,old', 'registration appends to the same moved List');
assert(source.dispose() === undefined && moved.dispose() === undefined, 'idempotent disposed stacks');
var value = Object.create(null);
try {new Proxy(new DisposableStack(), {}).use(value); throw new Error('missing brand error');}
catch(error) {assert(error instanceof TypeError, 'Proxy does not forward private stack brand');}
"#,
    );
}

#[test]
fn disposal_is_lifo_reentrant_and_does_not_assimilate_normal_returns() {
    assert_stack(
        r#"
var stack = new DisposableStack(), count = 0, first = {}, second = {}, captured;
for (var i=0;i<37;i++) {
    stack.adopt(i, function(index) {'use strict'; assert(this === undefined, 'adopt receiver'); assert(index === 36-count, 'grown List LIFO'); count++;});
}
assert(stack.use(null) === null && stack.use(undefined) === undefined, 'nullish use does not register');
stack.defer(function() {'use strict'; assert(this === undefined && stack.disposed, 'defer receiver and disposed state');
    stack.dispose();
    return Object.defineProperty({}, 'then', {get: function() {throw new Error('sync result must not be assimilated');}});
});
stack.dispose();
assert(count === 37 && stack.disposed, 'every entry consumed once');
try {stack.defer(3); throw new Error('missing disposed failure');}
catch(error) {assert(error instanceof ReferenceError, 'state failure precedes callability');}
var failing = new DisposableStack();
failing.defer(function() {throw first;});
failing.defer(function() {throw second;});
try {failing.dispose(); throw new Error('missing disposal failure');}
catch(error) {captured=error;}
assert(captured instanceof SuppressedError && captured.error === first && captured.suppressed === second, 'whole LIFO error/suppressed values');
assert(failing.disposed && failing.dispose() === undefined, 'failure still consumes the capability');
"#,
    );
}

#[test]
fn move_and_constructor_use_the_selected_realms_and_brand_only_completed_instances() {
    assert_stack(
        r#"
var other = __lilaCreateRealm().global, source = new DisposableStack(), token = {};
source.adopt(token, function(value) {assert(value === token, 'moved whole captured argument');});
var moved = other.DisposableStack.prototype.move.call(source);
assert(Object.getPrototypeOf(moved) === other.DisposableStack.prototype && source.disposed, 'move selects executing builtin Realm');
moved.dispose();
function Different() {}
var created = Reflect.construct(DisposableStack, [], Different);
assert(Object.getPrototypeOf(created) === Different.prototype, 'constructor observes newTarget prototype');
assert(DisposableStack.prototype.use.call(created, null) === null, 'completed constructor carries actual brand');
try {DisposableStack.prototype.dispose.call(DisposableStack.prototype); throw new Error('prototype brand');}
catch(error) {assert(error instanceof TypeError, 'prototype has no resource stack slots');}
try {other.DisposableStack.prototype.use.call(Object.create(other.DisposableStack.prototype), null); throw new Error('forged brand');}
catch(error) {assert(error instanceof other.TypeError && !(error instanceof TypeError), 'borrowed native error Realm');}
"#,
    );
}
