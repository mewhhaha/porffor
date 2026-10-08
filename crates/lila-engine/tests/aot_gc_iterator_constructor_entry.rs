use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_constructor(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for strict in [false, true] {
        let source = format!(
            "{}var $262={{createRealm:__lilaCreateRealm}};function assert(v,m){{if(!v)throw new Error(m);}}\n{source}",
            if strict { "\"use strict\";\n" } else { "" },
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
            .expect("Iterator constructor control uses Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion,
        );
    }
}

#[test]
fn actual_active_identity_rejects_before_prototype_get() {
    assert_constructor(
        r#"
var saved=Iterator, log=[];
var target=new Proxy(saved,{get:function(t,k,r){if(k==='prototype')log.push(k);return Reflect.get(t,k,r);}});
try{saved();throw new Error('call accepted');}catch(e){assert(e instanceof TypeError,'undefined NewTarget');}
try{new saved();throw new Error('abstract accepted');}catch(e){assert(e instanceof TypeError,'active NewTarget');}
var result=Reflect.construct(saved,[],target);
assert(Object.getPrototypeOf(result)===saved.prototype && log.join(',')==='prototype','proxy NewTarget is distinct');
Iterator=function Replacement(){};
try{new saved();throw new Error('global replacement accepted');}catch(e){assert(e instanceof TypeError,'captured actual identity');}
class Child extends saved { next(){return {value:17,done:false};} }
var child=new Child();assert(Object.getPrototypeOf(child)===Child.prototype && child.next().value===17,'subclass prototype');
"#,
    );
}

#[test]
fn prototype_get_retains_whole_throw_and_actual_object_kind() {
    assert_constructor(
        r#"
var token=Symbol('prototype'), log=[];
function Target(){}
var target=new Proxy(Target,{get:function(t,k,r){if(k==='prototype'){log.push(k);throw token;}return Reflect.get(t,k,r);}});
try{Reflect.construct(Iterator,[],target);throw new Error('throw lost');}catch(e){assert(e===token,'original whole throw');}
assert(log.join(',')==='prototype','Get once');
var prototype=new Map([[1,2]]);
var actual=new Proxy(Target,{get:function(t,k,r){if(k==='prototype')return prototype;return Reflect.get(t,k,r);}});
var result=Reflect.construct(Iterator,[],actual);
assert(Object.getPrototypeOf(result)===prototype && Map.prototype.get.call(prototype,1)===2,'complete semantic prototype');
var revoked=Proxy.revocable(Target,{});revoked.revoke();
try{Reflect.construct(Iterator,[],revoked.proxy);throw new Error('revocation lost');}catch(e){assert(e instanceof TypeError,'revoked constructor abrupt');}
"#,
    );
}

#[test]
fn primitive_prototype_uses_new_target_realm_saved_intrinsic() {
    assert_constructor(
        r#"
var realm=$262.createRealm(), other=realm.global, Foreign=other.Iterator, fallback=Foreign.prototype;
var foreignTarget=new other.Proxy(other.Proxy,{get:function(t,k,r){if(k==='prototype')return 3;return Reflect.get(t,k,r);}});
var result=Reflect.construct(Iterator,[],foreignTarget);
assert(Object.getPrototypeOf(result)===fallback,'NewTarget Realm fallback');
other.Iterator=function Replacement(){};
var again=Reflect.construct(Iterator,[],foreignTarget);
assert(Object.getPrototypeOf(again)===fallback,'saved Realm intrinsic survives replacement');
try{Reflect.construct(Foreign,[],Foreign);throw new Error('foreign abstract accepted');}catch(e){assert(e instanceof other.TypeError && !(e instanceof TypeError),'active function error Realm');}
var cross=Reflect.construct(Foreign,[],Iterator);
assert(Object.getPrototypeOf(cross)===Iterator.prototype,'different actual callable may construct');
"#,
    );
}
