use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};
fn assert_sparse(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for strict in [false, true] {
        let source = format!(
            "{}function assert(v,m){{if(!v)throw new Error(m);}}\n{source}",
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
            .expect("sparse Array controls use actual Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
    }
}
#[test]
fn largest_index_and_logical_length_do_not_allocate_holes() {
    assert_sparse(
        r#"
var token={},symbol=Symbol('v'),a=new Array(4294967295);assert(a.length===4294967295 && Reflect.ownKeys(a).join(',')==='length','length independent from storage');
a[4294967294]=token;a[17]=symbol;a[4294967295]=7n;assert(a.length===4294967295 && a[4294967294]===token && a[17]===symbol,'largest index whole references');
assert(Object.getOwnPropertyNames(a).join(',')==='17,4294967294,length,4294967295','ascending occupied indices then named order');delete a[4294967294];assert(a.length===4294967295 && !(4294967294 in a),'deletion preserves length');
a.length=0;assert(!(17 in a) && a[4294967295]===7n && a.length===0,'shrink occupied indices only');
"#,
    );
}
#[test]
fn complete_accessors_and_nonconfigurable_shrink_rollback_survive_storage() {
    assert_sparse(
        r#"
var token={},setterValue,sym=Symbol('s'),a=[];Object.defineProperty(a,'1000000',{get:function(){return token;},set:function(v){setterValue=v;},enumerable:true,configurable:true});a[1000000]=sym;assert(a[1000000]===token && setterValue===sym && a.length===1000001,'complete accessor descriptor');
Object.defineProperty(a,'500000',{value:7n,writable:false,enumerable:false,configurable:false});a[900000]={};var ok=Reflect.defineProperty(a,'length',{value:1,writable:false});assert(ok===false && a.length===500001 && !(900000 in a) && !(1000000 in a) && a[500000]===7n,'descending deletion before first failure');assert(Object.getOwnPropertyDescriptor(a,'length').writable===false,'final readonly length retained on failure');
assert(Reflect.defineProperty(a,'500000',{value:7n}) && !Reflect.defineProperty(a,'500000',{value:8n}),'retained complete compatibility check');
"#,
    );
}
#[test]
fn growth_replacement_and_unlink_keep_unique_sorted_keys() {
    assert_sparse(
        r#"
var token={},a=[];for(var i=0;i<128;i++){a[i*65537]=token;}for(var i=0;i<128;i+=2){delete a[i*65537];}for(var i=1;i<128;i+=2){a[i*65537]=7n;}
var names=Object.keys(a);assert(names.length===64,'replacement does not duplicate count');for(var i=0;i<64;i++){assert(names[i]===''+((i*2+1)*65537) && a[names[i]]===7n,'occupied keys sorted');}
for(var i=1;i<128;i+=2){delete a[i*65537];}assert(Object.keys(a).length===0 && a.length===127*65537+1,'nodes unlink while logical length remains');a[4294967294]=token;assert(Object.keys(a).join(',')==='4294967294' && a[4294967294]===token,'reuse empty storage');
"#,
    );
}
#[test]
fn missing_own_indices_preserve_inheritance_proxy_and_descriptor_identity() {
    assert_sparse(
        r#"
var token={},a=[];a.length=1000001;var proto=Object.create(Array.prototype);proto[999999]=token;Object.setPrototypeOf(a,proto);assert(a[999999]===token && !Object.prototype.hasOwnProperty.call(a,'999999'),'missing sparse own property falls through prototype');
var log=[],p=new Proxy(a,{getOwnPropertyDescriptor:function(t,k){log.push(k);return Reflect.getOwnPropertyDescriptor(t,k);}});assert(Object.getOwnPropertyDescriptor(p,'999999')===undefined && log.join(',')==='999999','proxy descriptor observes missing own');
a[999999]=token;var d=Object.getOwnPropertyDescriptor(a,'999999');assert(d.value===token && d.writable && d.enumerable && d.configurable,'whole own descriptor');assert(Reflect.ownKeys(a).join(',')==='999999,length','inherited index absent from own keys');
"#,
    );
}
#[test]
fn native_enumerable_and_regexp_results_publish_complete_descriptors() {
    assert_sparse(
        r#"
var token={},source={a:token,b:7n},entries=Object.entries(source);assert(entries[0][0]==='a' && entries[0][1]===token && entries[1][1]===7n,'native enumerable result Array');
var match=/a(.)/.exec('ab');assert(match[0]==='ab' && match[1]==='b','RegExp native result Array');var d=Object.getOwnPropertyDescriptor(match,'1');assert(d.value==='b' && d.writable && d.enumerable && d.configurable,'complete native descriptor');
"#,
    );
}

#[test]
fn fresh_literals_keep_holes_definition_order_and_source_realm_without_public_hooks() {
    assert_sparse(
        r#"
var token={},symbol=Symbol('literal'),inherited={};
var prototype=Array.prototype;
var oldZero=Object.getOwnPropertyDescriptor(prototype,'0');
var oldOne=Object.getOwnPropertyDescriptor(prototype,'1');
var originalDefine=Reflect.defineProperty,setterCalls=0,trace='';
function element(name,value){trace+=name;return value;}
function fail(){trace+='throw';throw token;}
try {
    Object.defineProperty(prototype,'0',{get:function(){return inherited;},set:function(){setterCalls++;},configurable:true});
    Object.defineProperty(prototype,'1',{value:inherited,writable:true,configurable:true});
    Reflect.defineProperty=function(){throw new Error('public Reflect must not initialize literals');};
    var literal=[element('first',token),,element('last',symbol),undefined];
    var first=Object.getOwnPropertyDescriptor(literal,'0');
    var last=Object.getOwnPropertyDescriptor(literal,'2');
    var explicitUndefined=Object.getOwnPropertyDescriptor(literal,'3');
    var length=Object.getOwnPropertyDescriptor(literal,'length');
    assert(trace==='firstlast' && setterCalls===0,'left-to-right own definitions bypass inherited setters and public Reflect');
    assert(first.value===token && first.writable && first.enumerable && first.configurable,'first whole value and standard attributes');
    assert(last.value===symbol && last.writable && last.enumerable && last.configurable,'Symbol value remains rooted across evaluations');
    assert(Object.getOwnPropertyDescriptor(literal,'1')===undefined && literal[1]===inherited,'elision remains absent and inherits');
    assert(explicitUndefined.value===undefined && explicitUndefined.writable && explicitUndefined.enumerable && explicitUndefined.configurable,'own undefined remains present');
    assert(length.value===4 && length.writable && !length.enumerable && !length.configurable,'logical length includes holes with standard attributes');
    trace='';var assigned='unchanged';
    try {assigned=[element('before',token),fail(),element('after',symbol)];throw new Error('missing element throw');}
    catch(error){assert(error===token && trace==='beforethrow' && assigned==='unchanged','original abrupt element prevents later evaluation and output publication');}
} finally {
    Reflect.defineProperty=originalDefine;
    if(oldZero===undefined){delete prototype[0];}else{Object.defineProperty(prototype,'0',oldZero);}
    if(oldOne===undefined){delete prototype[1];}else{Object.defineProperty(prototype,'1',oldOne);}
}
var other=__lilaCreateRealm().global,originalArray=Array;
try {
    Array=other.Array;
    function make(){return [token,,symbol];}
    var actual=make.call(other);
    assert(Object.getPrototypeOf(actual)===originalArray.prototype && Object.getPrototypeOf(actual)!==other.Array.prototype,'source Realm intrinsic survives replaced constructor and foreign receiver');
    assert(actual[0]===token && actual[2]===symbol && !Object.hasOwn(actual,'1'),'fresh private roots and holes survive Realm work');
} finally {Array=originalArray;}
"#,
    );
}
