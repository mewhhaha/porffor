use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_async(source: &str, marker: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for strict in [false, true] {
        let source = format!(
            "{}var $262={{createRealm:__lilaCreateRealm}};function assert(v,m){{if(!v)throw new Error(m);}}\n{source}",
            if strict { "\"use strict\";\n" } else { "" }
        );
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
            .expect("native async iterator disposal compiles through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(marker.into())]
        );
    }
}

#[test]
fn empty_call_list_and_whole_thenable_values_keep_method_promise_realm() {
    assert_async(
        r#"
var P=Promise,entry=(async function*(){})()[Symbol.asyncDispose];
var foreign=$262.createRealm(),other=foreign.global;
var foreignMethod=foreign.evalScript('(async function*(){})()[Symbol.asyncDispose]');
var entryPrototype=P.prototype,foreignPrototype=other.Promise.prototype;
other.Promise=function(){throw 'mutable global Promise';};
function go(method,prototype){
 var log=[],reads=0,receiver={};
 Object.defineProperty(receiver,'return',{get:function(){reads++;log.push('get');return new Proxy(function(){throw 'target bypassed';},{apply:function(target,self,args){assert(self===receiver && args.length===0,'empty argument List and receiver');log.push('call');return {then:function(resolve){log.push('then');resolve(Symbol('whole awaited value'));}};}});}});
 var promise=method.call(receiver);
 assert(Object.getPrototypeOf(promise)===prototype,'canonical defining-Realm Promise');
 return promise.then(function(value){assert(value===undefined && reads===1 && log.join(',')==='get,call,then','discard whole result after Await');});
}
P.all([go(entry,entryPrototype),go(foreignMethod,foreignPrototype)]).then(function(){print('async-dispose-call-realm');});
"#,
        "async-dispose-call-realm",
    );
}

#[test]
fn get_call_and_synchronous_promise_resolve_failures_reject_the_outer_promise() {
    assert_async(
        r#"
var P=Promise,method=(async function*(){})()[Symbol.asyncDispose],marker=Symbol('original'),jobs=[];
function rejected(receiver,expected){
 var promise;
 try {promise=method.call(receiver);} catch(error){throw 'asyncDispose escaped synchronously';}
 return promise.then(function(){throw 'expected rejection';},function(reason){assert(reason===expected,'whole original rejection');});
}
var getter={get return(){throw marker;}};
jobs.push(rejected(getter,marker));
jobs.push(rejected({return:new Proxy(function(){},{apply:function(){throw marker;}})},marker));
var reads=0,returned=P.resolve(7n);
Object.defineProperty(returned,'constructor',{get:function(){reads++;throw marker;}});
jobs.push(rejected({return:function(){assert(arguments.length===0,'empty Call List');return returned;}},marker).then(function(){assert(reads===1,'synchronous PromiseResolve constructor Get');}));
jobs.push(rejected({return:function(){return {get then(){throw marker;}};}},marker));
for (const receiver of [null,undefined,{return:1}]) {
 var promise=method.call(receiver);
 jobs.push(promise.then(function(){throw 'expected TypeError';},function(error){assert(Object.getPrototypeOf(error)===TypeError.prototype,'builtin error Realm');}));
}
for (const receiver of [{},{return:null},{return:function(){return 7n;}}]) {
 jobs.push(method.call(receiver).then(function(value){assert(value===undefined,'missing or primitive return fulfills undefined');}));
}
P.all(jobs).then(function(){
 var reads=0;
 Object.defineProperty(Number.prototype,'return',{configurable:true,get:function(){'use strict';assert(this===5,'GetV preserves original primitive receiver');reads++;return null;}});
 return method.call(5).then(function(value){delete Number.prototype.return;assert(reads===1 && value===undefined,'single primitive lookup');print('async-dispose-abrupt-cutoffs');});
});
"#,
        "async-dispose-abrupt-cutoffs",
    );
}
