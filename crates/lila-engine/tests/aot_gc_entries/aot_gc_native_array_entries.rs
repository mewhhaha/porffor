use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};
fn assert_array(source: &str) {
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
            .expect("native Array controls use actual Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
    }
}
#[test]
fn callbacks_preserve_live_holes_inheritance_and_whole_values() {
    assert_array(
        r#"
var token={},sym=Symbol('v'),proto={1:sym},o=Object.create(proto);o.length=4;o[0]=token;o[3]=7n;var seen=[];
var mapped=Array.prototype.map.call(o,function(v,i,a){seen.push(i);assert(a===o,'original receiver');if(i===0)o[2]=token;return v;});
assert(seen.join(',')==='0,1,2,3' && mapped[0]===token && mapped[1]===sym && mapped[2]===token && mapped[3]===7n,'live Has/Get and rooted values');
var hole=[,token,,sym];seen=[];assert(hole.find(function(v,i){seen.push(i);return i===1;})===token && seen.join(',')==='0,1','find visits hole');
seen=[];assert(hole.every(function(v,i){seen.push(i);return false;})===false && seen.join(',')==='1','every skips hole and stops');
var filtered=hole.filter(function(){return true;});assert(filtered.length===2 && filtered[0]===token && filtered[1]===sym,'filter whole values');
var acc={};assert([token,sym].reduce(function(a){return a;},acc)===acc,'reduce retains accumulator identity');
"#,
    );
}
#[test]
fn sorting_is_stable_and_admits_comparator_before_length() {
    assert_array(
        r#"
var a={k:1},b={k:0},c={k:1},items=[a,,undefined,b,c];
assert(items.sort(function(x,y){return x.k-y.k;})===items && items[0]===b && items[1]===a && items[2]===c && items[3]===undefined && !(4 in items),'stable values undefined then holes');
var x={toString:function(){return 'b';}},y={toString:function(){return 'a';}},sorted=[x,undefined,,y].toSorted();assert(sorted[0]===y && sorted[1]===x && sorted[2]===undefined && sorted[3]===undefined && 3 in sorted,'copy reads holes');
var error={};try{[2,1].sort(function(){throw error;});throw new Error('comparator');}catch(e){assert(e===error,'whole comparator Throw');}
var touched=0,o={get length(){touched++;return 0;}};try{Array.prototype.sort.call(o,3);throw new Error('bad compare');}catch(e){assert(e instanceof TypeError && touched===0,'compare admission precedes length');}
var called=0,v=[2,1];v.sort=function(){called++;return 9;};assert(v.sort()===9 && called===1,'own method remains actual Call');
"#,
    );
}
#[test]
fn concat_observes_spreadability_then_length_then_has_and_get() {
    assert_array(
        r#"
var token={},symbol=Symbol('s'),log=[],source={0:token,2:symbol};Object.defineProperty(source,Symbol.isConcatSpreadable,{get:function(){log.push('spread');return true;}});Object.defineProperty(source,'length',{get:function(){log.push('length');return 3;}});
var proxy=new Proxy(source,{has:function(t,k){log.push('has:'+k);return Reflect.has(t,k);},get:function(t,k,r){if(k==='0'||k==='2')log.push('get:'+k);return Reflect.get(t,k,r);}});
var out=[7n].concat(proxy);assert(out.length===4 && out[0]===7n && out[1]===token && !(2 in out) && out[3]===symbol,'whole sparse concatenation');assert(log.join(',')==='spread,length,has:0,get:0,has:1,has:2,get:2','spread protocol order');
var array=[token];array[Symbol.isConcatSpreadable]=false;assert([].concat(array)[0]===array,'false preserves wrapper identity');
"#,
    );
}
#[test]
fn splice_and_copy_methods_keep_their_distinct_hole_protocol() {
    assert_array(
        r#"
var token={},sym=Symbol('s'),a=[token,,7n,sym],removed=a.splice(1,2,sym,token,4n);assert(removed.length===2 && !(0 in removed) && removed[1]===7n && a.length===5 && a[1]===sym && a[2]===token && a[3]===4n && a[4]===sym,'splice sparse deletion and backwards insertion');
a=[token,,sym];var copy=a.toSpliced(1,1,7n);assert(copy.length===3 && copy[0]===token && copy[1]===7n && copy[2]===sym && !(1 in a),'copy skips deleted slot');
copy=[,token].toReversed();assert(copy[0]===token && copy[1]===undefined && 1 in copy,'reversed copy reads hole');
var reads=0,o={length:2,get 0(){reads++;return token;},get 1(){throw new Error('replacement read');}};copy=Array.prototype.with.call(o,1,sym);assert(copy[0]===token && copy[1]===sym && reads===1,'with skips replacement Get');
var p={length:3,0:token,2:sym};Array.prototype.copyWithin.call(p,1,0,2);assert(p[1]===token && !(2 in p),'copyWithin deletes hole destination');
"#,
    );
}
#[test]
fn join_locale_and_shared_to_string_keep_observable_calls() {
    assert_array(
        r#"
var log=[],token={toString:function(){log.push('value');return 'V';}},separator={toString:function(){log.push('separator');return '|';}};
assert([token,null,undefined].join(separator)==='V||' && log.join(',')==='separator,value','separator precedes elements');
log=[];var locales={},options={},value={get toLocaleString(){log.push('method');return function(l,o){assert(this===value && l===locales && o===options,'whole locale receiver and arguments');log.push('call');return {toString:function(){log.push('string');return 'L';}};};}};
assert([value,null].toLocaleString(locales,options)==='L,' && log.join(',')==='method,call,string','locale Get Call ToString');
var o={join:function(){assert(this===o,'boxed receiver');return 7n;}};assert(Array.prototype.toString.call(o)===7n,'generic join result whole');
var ta=new Uint8Array(0);ta.join=function(){return token;};assert(Array.prototype.toString===Uint8Array.prototype.toString && ta.toString()===token,'same callable identity with no branded toString');
"#,
    );
}
#[test]
fn constructor_from_of_and_is_array_use_actual_protocols() {
    assert_array(
        r#"
var token={},sym=Symbol('v');assert(Array(3).length===3 && !(0 in Array(3)),'length-only constructor');var a=Array.of(token,sym,7n);assert(a.length===3 && a[0]===token && a[1]===sym && a[2]===7n,'of values');
var closed=0,iterator={next:function(){return {value:token,done:false};},return:function(){closed++;return {};}};var iterable={};iterable[Symbol.iterator]=function(){return iterator;};var error={};try{Array.from(iterable,function(){throw error;});throw new Error('mapper');}catch(e){assert(e===error && closed===1,'Array.from mapper Throw closes once');}
var o={length:2,0:token};a=Array.from(o);assert(a[0]===token && a[1]===undefined && 1 in a,'arraylike reads holes');
var revoked=Proxy.revocable([],{});assert(Array.isArray(revoked.proxy),'actual proxy target');revoked.revoke();try{Array.isArray(revoked.proxy);throw new Error('revoked');}catch(e){assert(e instanceof TypeError,'revoked Array.isArray');}
"#,
    );
}
#[test]
fn typed_array_set_snapshots_overlap_and_keeps_conversion_order() {
    assert_array(
        r#"
var buffer=new ArrayBuffer(8),bytes=new Uint8Array(buffer),target=new Uint16Array(buffer);bytes.set([1,2,3,4,5,6,7,8]);target.set(bytes.subarray(0,3),1);assert(target[1]===1 && target[2]===2 && target[3]===3,'unlike overlapping byte snapshot precedes writes');
var values=new Uint8Array([1,2,3,4]);values.set(values.subarray(0,3),1);assert(values.join(',')==='1,1,2,3','same-kind overlap');
var log=[],source={get length(){log.push('length');return 2;},get 0(){log.push('get0');return {valueOf:function(){log.push('value0');return 9;}};},get 1(){log.push('get1');return 8;}};new Uint8Array(2).set(source);assert(log.join(',')==='length,get0,value0,get1','ordered arraylike conversion');
var immutable=new ArrayBuffer(0).transferToImmutable(),view=new Uint8Array(immutable),coercions=0;try{view.fill({valueOf:function(){coercions++;return 0;}});throw new Error('immutable');}catch(e){assert(e instanceof TypeError && coercions===0,'immutable write admission even empty');}
"#,
    );
}
#[test]
fn flat_and_search_keep_live_operations_and_whole_throw() {
    assert_array(
        r#"
var token={},sym=Symbol('s'),a=[token,,[sym,,[7n]]];var flat=a.flat(2);assert(flat.length===3 && flat[0]===token && flat[1]===sym && flat[2]===7n,'flat strong nested roots');
var log=[],o={length:3,0:token,2:sym},p=new Proxy(o,{has:function(t,k){log.push('has:'+k);return Reflect.has(t,k);},get:function(t,k,r){log.push('get:'+k);return Reflect.get(t,k,r);}});
assert(Array.prototype.indexOf.call(p,sym)===2 && log.join(',')==='get:length,has:0,get:0,has:1,has:2,get:2','indexOf Has before Get');log=[];assert(Array.prototype.includes.call(p,undefined) && log.join(',')==='get:length,get:0,get:1','includes reads hole');
var error={},from={valueOf:function(){throw error;}};assert([].indexOf(token,from)===-1,'empty before fromIndex');try{[token].at(from);throw new Error('at');}catch(e){assert(e===error,'whole coercion Throw');}
"#,
    );
}
