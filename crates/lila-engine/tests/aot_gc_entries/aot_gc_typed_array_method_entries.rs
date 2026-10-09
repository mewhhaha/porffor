use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};
fn assert_typed_methods(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for strict in [false, true] {
        let source = format!(
            "{}var $262={{createRealm:__lilaCreateRealm,gc:gc,detachArrayBuffer:__lilaDetachArrayBuffer}};function assert(v,m){{if(!v)throw new Error(m);}}\n{source}",
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
            .expect("TypedArray control uses Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
    }
}
#[test]
fn all_element_domains_use_same_type_copying_and_numeric_sort() {
    assert_typed_methods(
        r#"
var constructors=[Float64Array,Float32Array,Float16Array,Int32Array,Int16Array,Int8Array,Uint32Array,Uint16Array,Uint8Array,Uint8ClampedArray,BigInt64Array,BigUint64Array];
for(var i=0;i<constructors.length;i++){
 var C=constructors[i], big=C===BigInt64Array||C===BigUint64Array, values=big?[3n,1n,2n]:[3,1,2], a=new C(values);
 Object.defineProperty(a,'constructor',{get:function(){throw new Error('constructor read');}});
 var r=a.toReversed(), w=a.with(1,big?9n:9), sorted=a.toSorted();
 assert(Object.getPrototypeOf(r)===C.prototype && Object.getPrototypeOf(w)===C.prototype && Object.getPrototypeOf(sorted)===C.prototype,'intrinsic same type');
 assert(r.join(',')==='2,1,3' && w.join(',')==='3,9,2' && sorted.join(',')==='1,2,3' && a.join(',')==='3,1,2','fresh copies');
 assert(a.reverse()===a && a.join(',')==='2,1,3','reverse identity');
 assert(a.sort()===a && a.join(',')==='1,2,3','in-place numeric sort');
}
"#,
    );
}
#[test]
fn copywithin_preserves_nan_bytes_and_both_overlap_directions() {
    assert_typed_methods(
        r#"
var buffer=new ArrayBuffer(32), data=new DataView(buffer);
data.setUint32(0,0x89abcdef,true);data.setUint32(4,0x7ff81234,true);
var a=new Float64Array(buffer);assert(a.copyWithin(1,0,1)===a,'copy identity');
assert(data.getUint32(8,true)===0x89abcdef && data.getUint32(12,true)===0x7ff81234,'exact NaN encoding');
var back=new Uint8Array([1,2,3,4,5,6]);back.copyWithin(1,0,5);assert(back.join(',')==='1,1,2,3,4,5','backwards overlap');
var forward=new Uint8Array([1,2,3,4,5,6]);forward.copyWithin(0,1,6);assert(forward.join(',')==='2,3,4,5,6,6','forwards overlap');
var shared=new SharedArrayBuffer(8), bytes=new Uint8Array(shared);bytes.set([1,2,3,4,5,6]);bytes.copyWithin(1,0,5);
assert(bytes.join(',')==='1,1,2,3,4,5,0,0','shared byte owner');
"#,
    );
}
#[test]
fn copywithin_coerces_once_then_reobserves_live_bounds() {
    assert_typed_methods(
        r#"
var buffer=new ArrayBuffer(4,{maxByteLength:8}), a=new Uint8Array(buffer);a.set([1,2,3,4]);var log=[];
a.copyWithin({valueOf:function(){log.push('target');return 1;}},{valueOf:function(){log.push('start');return 0;}},{valueOf:function(){log.push('end');buffer.resize(2);return 4;}});
assert(log.join(',')==='target,start,end' && a.join(',')==='1,1','snapshot indices and longest prefix');
var token={};try{a.copyWithin(0,{valueOf:function(){throw token;}});throw new Error('missing throw');}catch(e){assert(e===token,'original coercion throw');}
var fixed=new Uint8Array(buffer,0,2);fixed.copyWithin(2,{valueOf:function(){buffer.resize(0);return 2;}});
assert(fixed.length===0,'zero count does not add a bounds failure');
"#,
    );
}
#[test]
fn with_orders_coercions_and_validates_against_current_length() {
    assert_typed_methods(
        r#"
var buffer=new ArrayBuffer(3,{maxByteLength:6}), a=new Uint8Array(buffer);a.set([1,2,3]);var log=[];
try{a.with({valueOf:function(){log.push('index');return -1;}},{valueOf:function(){log.push('value');buffer.resize(1);return 9;}});throw new Error('missing range');}catch(e){assert(e instanceof RangeError,'post-coercion current validity');}
assert(log.join(',')==='index,value','coercion order');
var grown=new ArrayBuffer(2,{maxByteLength:4}), view=new Uint8Array(grown);view.set([4,5]);
var copied=view.with(2,{valueOf:function(){grown.resize(3);return 8;}});assert(copied.join(',')==='4,5','old output length and grown current validity');
var marker={};try{view.with(Infinity,{valueOf:function(){throw marker;}});throw new Error('missing conversion throw');}catch(e){assert(e===marker,'value coercion precedes range error');}
var big=new BigInt64Array([1n,2n]);assert(big.with(-1,3n).join(',')==='1,3','BigInt value');
try{big.with(0,1);throw new Error('accepted Number');}catch(e){assert(e instanceof TypeError,'ToBigInt rejects Number');}
"#,
    );
}
#[test]
fn default_sort_keeps_numeric_bigint_zero_and_nan_order() {
    assert_typed_methods(
        r#"
var a=new Float64Array([NaN,10,2,0,-0,-2,NaN]);a.sort();
assert(a[0]===-2 && 1/a[1]===-Infinity && 1/a[2]===Infinity && a[3]===2 && a[4]===10 && a[5]!==a[5] && a[6]!==a[6],'numeric zero/NaN ordering');
var big=new BigInt64Array([9223372036854775807n,-9223372036854775808n,0n,-1n,1n]);
assert(big.toSorted().join(',')==='-9223372036854775808,-1,0,1,9223372036854775807','signed exact magnitude comparison');
var unsigned=new BigUint64Array([18446744073709551615n,0n,9223372036854775808n,1n]);
assert(unsigned.toSorted().join(',')==='0,1,9223372036854775808,18446744073709551615','unsigned exact magnitude comparison');
var tied=new Int32Array([21,11,22,12]);tied.sort(function(x,y){return Math.floor(x/10)-Math.floor(y/10);});assert(tied.join(',')==='11,12,21,22','stable equivalence');
var nan=new Int32Array([4,3,2,1]);nan.sort(function(){return NaN;});assert(nan.join(',')==='4,3,2,1','NaN comparator means stable equality');
"#,
    );
}
#[test]
fn sort_snapshots_before_callbacks_and_preserves_whole_throws() {
    assert_typed_methods(
        r#"
var a=new Int32Array([3,1,2]), first=true;
var sorted=a.toSorted(function(x,y){if(first){first=false;a.fill(9);$262.gc();}return x-y;});
assert(sorted.join(',')==='1,2,3' && a.join(',')==='9,9,9','snapshot before observable comparison');
var token={}, b=new Int32Array([3,1,2]);try{b.sort(function(){return {valueOf:function(){throw token;}};});throw new Error('missing throw');}catch(e){assert(e===token,'whole comparison ToNumber throw');}
assert(b.join(',')==='3,1,2','no writeback before completed comparison');
var buffer=new ArrayBuffer(12), c=new Int32Array(buffer);c.set([3,1,2]);var detached=false;
assert(c.sort(function(x,y){if(!detached){detached=true;$262.detachArrayBuffer(buffer);$262.gc();}return x-y;})===c && c.length===0,'detached writeback is suppressed');
var copyBuffer=new ArrayBuffer(12), d=new Int32Array(copyBuffer);d.set([3,1,2]);var did=false;
var copied=d.toSorted(function(x,y){if(!did){did=true;$262.detachArrayBuffer(copyBuffer);}return x-y;});assert(copied.join(',')==='1,2,3','copy retains rooted snapshot');
"#,
    );
}
#[test]
fn concrete_brand_and_defining_realm_survive_borrowing() {
    assert_typed_methods(
        r#"
var reads=0, fake=new Proxy(new Float64Array([1,2]),{get:function(){reads++;throw new Error('proxy Get');}});
for(var method of ['reverse','copyWithin','toReversed','sort','toSorted','with']){
 try{Float64Array.prototype[method].call(fake,0,1);throw new Error('accepted Proxy');}catch(e){assert(e instanceof TypeError,'concrete brand');}
}
assert(reads===0,'brand check has no property lookup');
var realm=$262.createRealm(), foreign=realm.evalScript('new Float64Array([3,1,2])');
var result=Float64Array.prototype.toReversed.call(foreign);assert(Object.getPrototypeOf(result)===Float64Array.prototype && result.join(',')==='2,1,3','active function Realm same type');
var method=realm.evalScript('Float64Array.prototype.toSorted');var back=method.call(new Float64Array([3,2,1]));
assert(Object.getPrototypeOf(back)===realm.global.Float64Array.prototype && back.join(',')==='1,2,3','foreign defining Realm');
"#,
    );
}
#[test]
fn immutable_mutators_reject_before_coercion_even_when_empty() {
    assert_typed_methods(
        r#"
var buffer=new ArrayBuffer(3), before=new Uint8Array(buffer);before.set([3,1,2]);var immutable=buffer.transferToImmutable(), a=new Uint8Array(immutable), calls=0;
try{a.copyWithin({valueOf:function(){calls++;return 0;}});throw new Error('accepted immutable copy');}catch(e){assert(e instanceof TypeError,'write admission');}
try{a.sort(function(){calls++;return 0;});throw new Error('accepted immutable sort');}catch(e){assert(e instanceof TypeError,'sort write admission');}
try{a.reverse();throw new Error('accepted immutable reverse');}catch(e){assert(e instanceof TypeError,'reverse write admission');}
assert(calls===0 && a.join(',')==='3,1,2','reject before callbacks/coercions');
assert(a.toReversed().join(',')==='2,1,3' && a.toSorted().join(',')==='1,2,3' && a.with(1,9).join(',')==='3,9,2','read copying methods');
var empty=new Uint8Array(new ArrayBuffer(0).transferToImmutable());
for(var name of ['reverse','copyWithin','sort']){try{empty[name]();throw new Error('accepted empty immutable receiver');}catch(e){assert(e instanceof TypeError,'empty write admission');}}
"#,
    );
}
