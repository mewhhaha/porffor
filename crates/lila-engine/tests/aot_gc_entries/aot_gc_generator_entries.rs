use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_generators(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    let source = format!("var $262={{createRealm:__lilaCreateRealm,gc:gc}};function assert(v,m) {{ if (!v) throw new Error(m); }}\n{source}");
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
        .expect("generator control uses Wasm AOT");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observed.completion
    );
}

#[test]
fn generators_distinguish_start_completion_and_whole_abrupt_entries() {
    assert_generators(
        r#"
var started=0, token={};
function* body(){started++; var value=yield 1; return value;}
var unopened=body(), completed=unopened.return(token);
assert(completed.done && completed.value===token && started===0,'Return before start');
assert(unopened.next(token).done && unopened.next().value===undefined,'permanent completion');
var thrown=body();
try {thrown.throw(token);throw new Error('missing whole Throw');}
catch(error){assert(error===token && started===0,'Throw before start');}
assert(thrown.next().done,'Throw permanently completes');
var live=body(), first=live.next(token);
assert(first.value===1 && !first.done && started===1,'first argument ignored');
var last=live.next(token);
assert(last.done && last.value===token && live.next().value===undefined,'resume whole Return');
function* empty(){started++;}
assert(empty().next().done && started===2,'normal fallthrough completes');
try {body().next.call(new Proxy(body(),{}));throw new Error('Proxy brand');}
catch(error){assert(error instanceof TypeError,'Proxy does not forward activation');}
"#,
    );
}

#[test]
fn generators_retain_finally_completions_gc_roots_and_executing_state() {
    assert_generators(
        r#"
var token={}, live, reentrant=0;
function* body(){
  try {
    try {live.next();throw new Error('missing executing failure');}
    catch(error){assert(error instanceof TypeError,'running state');reentrant++;}
    yield token;
  } finally {yield {cleanup:true};}
}
live=body();
var first=live.next();
assert(first.value===token && !first.done && reentrant===1,'reentrant failure preserves body');
$262.gc();
var cleanup=live.return(token);
assert(!cleanup.done && cleanup.value.cleanup,'Return enters finally');
$262.gc();
var last=live.next();
assert(last.done && last.value===token,'pending whole Return survives cleanup yield');
var thrown=body();live=thrown;thrown.next();
assert(!thrown.throw(token).done,'Throw enters finally');
try {thrown.next();throw new Error('lost pending Throw');}
catch(error){assert(error===token,'pending whole Throw survives cleanup yield');}
"#,
    );
}

#[test]
fn delegated_yields_return_the_original_result_and_read_done_once() {
    assert_generators(
        r#"
var order=[], token={}, count=0;
var result=new Proxy({get done(){order.push('done');return false;},
  get value(){order.push('value');return token;}},
  {get:function(t,k,r){order.push('get:'+String(k));return Reflect.get(t,k,r);}});
var iterator={next:function(value){order.push('next');count++;return count===1?result:{done:true,value:token};},
  [Symbol.iterator]:function(){return this;}};
function* delegate(){return yield* iterator;}
var generator=delegate(), first=generator.next();
assert(first===result && order.join(',')==='next,get:done,done','original result, no value Get');
$262.gc();
var last=generator.next(token);
assert(last.done && last.value===token && count===2,'retained delegate completes');
assert(generator.next().done && count===2,'terminal delegate is cleared');
var calls=0, closeToken={}, missingThrow={
  next:function(){return {value:1,done:false};},
  return:function(){calls++;throw closeToken;},
  [Symbol.iterator]:function(){return this;}
};
function* closing(){yield* missingThrow;}
var closed=closing();closed.next();
try {closed.throw(token);throw new Error('missing close Throw');}
catch(error){assert(error===closeToken && calls===1,'missing throw closes once');}
assert(closed.next().done && calls===1,'failed delegate terminal');
"#,
    );
}

#[test]
fn live_generator_results_use_the_saved_body_realm() {
    assert_generators(
        r#"
var realm=__lilaCreateRealm(), other=realm.global;
var factory=realm.evalScript('function* sequence(){yield 7;return 9;} sequence;');
var mainNext=Object.getPrototypeOf((function*(){})()).next, generator=factory();
var first=mainNext.call(generator);
assert(!first.done && first.value===7 && Object.getPrototypeOf(first)===other.Object.prototype,'live yield Realm');
var last=mainNext.call(generator);
assert(last.done && last.value===9 && Object.getPrototypeOf(last)===Object.prototype,'terminal caller Realm');
var completed=mainNext.call(generator);
assert(completed.done && Object.getPrototypeOf(completed)===Object.prototype,'already-completed builtin Realm');
"#,
    );
}

#[test]
fn async_generators_reject_brand_and_before_start_throw_with_intrinsic_promises() {
    assert_generators(
        r#"
var started=0, token={};
async function* body(){started++;yield 1;}
var live=body(), next=Object.getPrototypeOf(live).next, rejected;
try {rejected=next.call({});}catch(error){throw new Error('brand threw synchronously');}
assert(rejected instanceof Promise,'brand creates Promise');
rejected.then(function(){throw new Error('brand fulfilled');},function(error){
  assert(error instanceof TypeError,'brand rejection');
});
var unopened=body(), thrown=unopened.throw(token);
assert(thrown instanceof Promise && started===0,'Throw before start creates Promise');
thrown.then(function(){throw new Error('Throw fulfilled');},function(error){
  assert(error===token && started===0,'whole Throw and unopened body');
});
unopened.next().then(function(result){assert(result.done && result.value===undefined,'permanent async completion');});
var realm=__lilaCreateRealm(), other=realm.global;
var otherNext=realm.evalScript('Object.getPrototypeOf((async function*(){})()).next;');
var borrowed=otherNext.call(new Proxy(live,{}));
assert(borrowed instanceof other.Promise && !(borrowed instanceof Promise),'intrinsic capability Realm');
borrowed.then(function(){throw new Error('Proxy brand fulfilled');},function(error){
  assert(error instanceof other.TypeError && !(error instanceof TypeError),'borrowed brand error Realm');
});
"#,
    );
}

#[test]
fn async_generator_queued_return_awaits_once_and_retains_pending_finally_value() {
    assert_generators(
        r#"
var constructorGets=0, value=Promise.resolve(9);
Object.defineProperty(value,'constructor',{get:function(){constructorGets++;return Promise;}});
async function* body(){try {yield 1;}finally {yield 'cleanup';}}
var generator=body(), first=generator.next(), cleanup=generator.return(value), last=generator.next(), done=generator.next();
$262.gc();
first.then(function(result){assert(result.value===1 && !result.done,'first queued yield');});
cleanup.then(function(result){assert(result.value==='cleanup' && !result.done,'queued Return enters finally');});
last.then(function(result){assert(result.value===9 && result.done && constructorGets===1,'Return Await once and pending completion');});
done.then(function(result){assert(result.done && result.value===undefined,'tail drains permanently');});
var calls=0, unopened=body(), token={};
var before=unopened.return({then:function(resolve){calls++;resolve(token);}});
var after=unopened.next();
$262.gc();
before.then(function(result){assert(result.done && result.value===token && calls===1,'before-start Return assimilates once');});
after.then(function(result){assert(result.done && result.value===undefined && calls===1,'pending completed queue');});
"#,
    );
}

#[test]
fn await_setup_abrupt_is_synchronous_and_resumes_the_retained_generator() {
    assert_generators(
        r#"
var token={}, events=[], bad=Promise.resolve(1);
Object.defineProperty(bad,'constructor',{get:function(){events.push('get');throw token;}});
async function plain(){try{await bad;}catch(e){assert(e===token,'plain whole Throw');events.push('plain-catch');}return 7;}
var plainPromise=plain();events.push('plain-after');
assert(events.join(',')==='get,plain-catch,plain-after','Await setup catches before caller continues');
plainPromise.then(function(v){assert(v===7,'plain resolved after handled setup');});
events=[];
async function* yielding(){try{yield bad;}catch(e){assert(e===token,'Yield whole Throw');events.push('yield-catch');yield 8;}}
var generator=yielding(), first=generator.next();events.push('yield-after');
assert(events.join(',')==='get,yield-catch,yield-after','Yield setup resumes same invocation');
$262.gc();
first.then(function(out){
  assert(out.value===8 && !out.done,'caught Yield proceeds to another yield');
  return generator.next();
}).then(function(out){assert(out.done,'Yield generator finishes normally');});
async function* returning(){try{yield 1;}catch(e){assert(e===token,'Return whole Throw');events.push('return-catch');yield 9;}}
var returningGenerator=returning();
returningGenerator.next().then(function(){
  events=[];var result=returningGenerator.return(bad);events.push('return-after');
  assert(events.join(',')==='get,return-catch,return-after','Return setup enters body catch immediately');
  return result;
}).then(function(out){
  assert(out.value===9 && !out.done,'Return setup keeps active request');
  return returningGenerator.next();
}).then(function(out){assert(out.done,'handled Return setup does not escape native method');});
"#,
    );
}
