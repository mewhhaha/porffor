use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};
fn assert_mutation(source: &str) {
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
            .expect("mutation controls use actual Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
    }
}

#[test]
fn growth_uses_one_length_snapshot_and_whole_arguments() {
    assert_mutation(
        r#"
var log=[],length=0,token={},symbol=Symbol('s'),o={get length(){log.push('length');return {valueOf:function(){log.push('coerce');return 1;}};},set length(v){log.push('length-set:'+v);length=v;}};
assert(Array.prototype.push.call(o,token,symbol,7n)===4 && o[1]===token && o[2]===symbol && o[3]===7n && length===4,'whole ordered append');
assert(log.join(',')==='length,coerce,length-set:4','one cached length and final Set');
var seen=[],p={get length(){return 0;},set length(v){seen.push(v);}};
assert(Array.prototype.push.call(p)===0 && Array.prototype.unshift.call(p)===0 && seen.join(',')==='0,0','empty growth still Set length');
"#,
    );
}

#[test]
fn pop_get_delete_length_and_empty_order() {
    assert_mutation(
        r#"
var token={},log=[],target={length:2,1:token};
var p=new Proxy(target,{get:function(t,k,r){log.push('get:'+k);return Reflect.get(t,k,r);},deleteProperty:function(t,k){log.push('delete:'+k);return Reflect.deleteProperty(t,k);},set:function(t,k,v){log.push('set:'+k+':'+v);return Reflect.set(t,k,v,t);}});
assert(Array.prototype.pop.call(p)===token && target.length===1 && !(1 in target),'whole pop result');
assert(log.join(',')==='get:length,get:1,delete:1,set:length:1','pop protocol order');
log=[];target.length=0;assert(Array.prototype.pop.call(p)===undefined && log.join(',')==='get:length,set:length:0','empty pop still Set');
var error={},o={get length(){throw error;}};try{Array.prototype.pop.call(o);throw new Error('length');}catch(e){assert(e===error,'length Throw retained');}
"#,
    );
}

#[test]
fn shift_uses_has_get_and_preserves_sparse_inherited_values() {
    assert_mutation(
        r#"
var token={},proto={1:token},o=Object.create(proto);o.length=3;o[0]=7;o[2]=9;
assert(Array.prototype.shift.call(o)===7 && o.length===2 && o[0]===token && o[1]===9 && !(2 in o),'inherited source copied whole');
var log=[],target={length:3,0:1,2:3},p=new Proxy(target,{get:function(t,k,r){log.push('get:'+k);return Reflect.get(t,k,r);},has:function(t,k){log.push('has:'+k);return Reflect.has(t,k);},set:function(t,k,v){log.push('set:'+k);return Reflect.set(t,k,v,t);},deleteProperty:function(t,k){log.push('delete:'+k);return Reflect.deleteProperty(t,k);}});
assert(Array.prototype.shift.call(p)===1 && target[1]===3 && !(0 in target),'hole deletes destination');
assert(log.join(',')==='get:length,get:0,has:1,delete:0,has:2,get:2,set:1,delete:2,set:length','shift observable order');
"#,
    );
}

#[test]
fn unshift_moves_backwards_before_inserting_arguments() {
    assert_mutation(
        r#"
var token={},symbol=Symbol('v'),log=[],target={length:3,0:token,2:7n};
var p=new Proxy(target,{get:function(t,k,r){log.push('get:'+k);return Reflect.get(t,k,r);},has:function(t,k){log.push('has:'+k);return Reflect.has(t,k);},set:function(t,k,v){log.push('set:'+k);return Reflect.set(t,k,v,t);},deleteProperty:function(t,k){log.push('delete:'+k);return Reflect.deleteProperty(t,k);}});
assert(Array.prototype.unshift.call(p,symbol,2)===5 && target[0]===symbol && target[1]===2 && target[2]===token && !(3 in target) && target[4]===7n,'whole values and preserved hole');
assert(log.join(',')==='get:length,has:2,get:2,set:4,has:1,delete:3,has:0,get:0,set:2,set:0,set:1,set:length','backwards move then insertion');
"#,
    );
}

#[test]
fn failed_writes_and_deletes_preserve_partial_mutation() {
    assert_mutation(
        r#"
var token={},error={},o={length:1};Object.defineProperty(o,'2',{value:0,writable:false});
try{Array.prototype.push.call(o,token,3);throw new Error('readonly');}catch(e){assert(e instanceof TypeError && o[1]===token && o.length===1,'first write survives failure');}
o={length:1,0:token};Object.defineProperty(o,'0',{configurable:false});
try{Array.prototype.pop.call(o);throw new Error('delete');}catch(e){assert(e instanceof TypeError && o[0]===token && o.length===1,'failed delete prevents length Set');}
var target={length:2,0:1,1:2},p=new Proxy(target,{set:function(t,k,v){if(k==='0')throw error;return Reflect.set(t,k,v,t);}});
try{Array.prototype.shift.call(p);throw new Error('setter');}catch(e){assert(e===error && target.length===2 && target[1]===2,'original setter Throw');}
p=new Proxy({length:0},{set:function(){return false;}});try{Array.prototype.unshift.call(p);throw new Error('false');}catch(e){assert(e instanceof TypeError,'strict false Set throws');}
"#,
    );
}

#[test]
fn generic_receivers_and_defining_realm_errors() {
    assert_mutation(
        r#"
var methods=[Array.prototype.pop,Array.prototype.push,Array.prototype.shift,Array.prototype.unshift];
for(var i=0;i<methods.length;i++){try{methods[i].call(null);throw new Error('null');}catch(e){assert(e instanceof TypeError,'ToObject null');}try{methods[i].call('a');throw new Error('String');}catch(e){assert(e instanceof TypeError,'boxed String property rejection');}}
assert(Array.prototype.pop.call({length:-1})===undefined,'ToLength clamps negative');
var count=0,last=1,o={get length(){count++;return last;},set length(v){last=v;},0:5};assert(Array.prototype.shift.call(o)===5 && count===1 && last===0,'one length Get');
var realm=$262.createRealm(),other=realm.global,p=new Proxy({length:0},{set:function(){return false;}});
try{other.Array.prototype.push.call(p);throw new Error('foreign');}catch(e){assert(e instanceof other.TypeError && !(e instanceof TypeError),'defining Realm Set error');}
"#,
    );
}
