use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};
fn assert_helpers(source: &str) {
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
            .expect("iterator controls use the actual Wasm AOT backend");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
    }
}

#[test]
fn callback_admission_closes_before_cached_next() {
    assert_helpers(
        r#"
var token={}, log=[], source={get next(){log.push('next');return function(){return {done:true};};},get return(){log.push('return');return function(){log.push('close');throw token;};}};
try{Iterator.prototype.map.call(source,1);throw new Error('callback accepted');}catch(e){assert(e instanceof TypeError && e!==token,'callback original TypeError');}
assert(log.join(',')==='return,close','close precedes next Get');
var closed=0, p={next:function(){return {done:true};},return:function(){closed++;return {};}}; log=[];
try{Iterator.prototype.map.call(1,function(){});throw new Error('primitive');}catch(e){assert(e instanceof TypeError,'Object admission');}
try{Iterator.prototype.take.call(p,{valueOf:function(){throw token;}});throw new Error('coercion');}catch(e){assert(e===token && closed===1,'coercion Throw closes');}
try{Iterator.prototype.drop.call(p,NaN);throw new Error('NaN');}catch(e){assert(e instanceof RangeError && closed===2,'NaN closes');}
try{Iterator.prototype.take.call(p,9007199254740992);throw new Error('finite limit');}catch(e){assert(e instanceof RangeError && closed===3,'finite limit admission');}
"#,
    );
}

#[test]
fn map_filter_cache_whole_values_and_number_indices() {
    assert_helpers(
        r#"
var nextGets=0,n=0, token={}, values=[token,7n,Symbol('s')], indices=[], receivers=[];
var source={get next(){nextGets++;return function(){return n<3?{value:values[n++],done:false}:{done:true};};}};
var helper=Iterator.prototype.map.call(source,function(v,i){'use strict';indices.push(i);receivers.push(this);return v;});
Object.defineProperty(source,'next',{value:function(){throw new Error('cached next changed');}});
var a=helper.next(),b=helper.next(),c=helper.next();
assert(a.value===token && b.value===7n && c.value===values[2] && !a.done && helper.next().done,'whole values');
assert(nextGets===1 && indices.join(',')==='0,1,2' && receivers[0]===undefined && receivers[2]===undefined,'direct cache/counter/this');
var seen=[], f=[10,11,12,13].values().filter(function(v,i){seen.push(i);return v%2;});
assert(f.next().value===11 && f.next().value===13 && f.next().done && seen.join(',')==='0,1,2,3','filter counter counts inputs');
"#,
    );
}

#[test]
fn drop_skips_value_get_and_take_closes_at_resume() {
    assert_helpers(
        r#"
var reads=0,i=0,closed=0, source={next:function(){var done=i===3;return {done:done,get value(){reads++;return i++;}};},return:function(){closed++;return {};}};
var skipped=0, d=Iterator.prototype.drop.call({next:function(){skipped++;return {done:false,get value(){reads++;return skipped;}};}},2);
assert(d.next().value===3 && reads===1,'elisions do not Get value');
var x=0, t=Iterator.prototype.take.call({next:function(){return {value:++x,done:false};},return:function(){closed++;return {};}},1);
assert(t.next().value===1 && closed===0,'yield does not close immediately');
assert(t.next().done && closed===1 && t.next().done && closed===1,'following resume closes once');
var inf=[3,4].values().take(Infinity);assert(inf.next().value===3 && inf.next().value===4 && inf.next().done,'Infinity retained');
var zero=Iterator.prototype.take.call(source,-0);assert(zero.next().done && closed===2,'negative zero accepted');
"#,
    );
}

#[test]
fn step_abrupt_does_not_close_and_completes_helper() {
    assert_helpers(
        r#"
var token={}, close=0;
function check(next){var h=Iterator.prototype.map.call({next:next,return:function(){close++;return {}; }},function(v){return v;});try{h.next();throw new Error('expected');}catch(e){assert(e===token,'original step Throw');}assert(h.next().done && h.return().done,'completed state');}
check(function(){throw token;});
check(function(){return {get done(){throw token;}};});
check(function(){return {done:false,get value(){throw token;}};});
assert(close===0,'no Close after failed step/value');
var h=Iterator.prototype.map.call({next:3,return:function(){close++;return {}; }},function(v){return v;});
try{h.next();throw new Error('callable');}catch(e){assert(e instanceof TypeError,'deferred next callability');}
assert(h.next().done && close===0,'next TypeError completes');
"#,
    );
}

#[test]
fn callback_throw_closes_and_reentrant_errors_preserve_state() {
    assert_helpers(
        r#"
var token={}, discarded={}, log=[], h;
h=Iterator.prototype.map.call({next:function(){return {value:1,done:false};},return:function(){log.push('close');throw discarded;}},function(){throw token;});
try{h.next();throw new Error('callback');}catch(e){assert(e===token,'callback wins close Throw');}
assert(h.next().done && log.join(',')==='close','completed after abrupt');
var n=0, nested=0;
h=Iterator.prototype.map.call({next:function(){return n++<2?{value:n,done:false}:{done:true};}},function(v){try{h.next();}catch(e){assert(e instanceof TypeError,'next reentrancy');nested++;}try{h.return();}catch(e){assert(e instanceof TypeError,'return reentrancy');nested++;}return v*2;});
assert(h.next().value===2 && h.next().value===4 && h.next().done && nested===4,'caught nested errors leave outer generator usable');
"#,
    );
}

#[test]
fn return_before_start_and_active_return_have_distinct_lifecycle() {
    assert_helpers(
        r#"
var h,log=[],opened=0;
h=Iterator.prototype.map.call({next:function(){opened++;return {value:1,done:false};},return:function(){log.push(h.next().done);log.push(h.return().done);return {}; }},function(v){return v;});
var r=h.return(99);assert(r.done && r.value===undefined && log.join(',')==='true,true' && opened===0,'start return completed before close');
h=Iterator.prototype.map.call({next:function(){return {value:1,done:false};},return:function(){try{h.next();throw new Error('running');}catch(e){assert(e instanceof TypeError,'active return executing');}return {}; }},function(v){return v;});
assert(h.next().value===1 && h.return().done && h.next().done,'active close then completed');
"#,
    );
}

#[test]
fn flat_map_inner_acquisition_and_close_order() {
    assert_helpers(
        r#"
var log=[], outer={i:0,next:function(){return this.i++<1?{value:1,done:false}:{done:true};},return:function(){log.push('outer');return {};}};
var h=Iterator.prototype.flatMap.call(outer,function(){return {next:function(){return {value:5,done:false};},return:function(){log.push('inner');throw 17;}};});
assert(h.next().value===5,'nullish method direct inner');
try{h.return();throw new Error('inner Throw');}catch(e){assert(e===17,'inner Throw retained');}
assert(log.join(',')==='inner,outer' && h.next().done,'close inner then outer');
var token={}, closes=0, source={next:function(){return {value:1,done:false};},return:function(){closes++;return {};}};
h=Iterator.prototype.flatMap.call(source,function(){return {get next(){throw token;}};});
try{h.next();throw new Error('acquisition');}catch(e){assert(e===token && closes===1,'inner next Get failure closes outer');}
h=Iterator.prototype.flatMap.call(source,function(){return 'abc';});
try{h.next();throw new Error('primitive');}catch(e){assert(e instanceof TypeError && closes===2,'primitive mapped String rejected');}
"#,
    );
}

#[test]
fn flat_map_empty_inner_counter_and_step_failure() {
    assert_helpers(
        r#"
var seen=[], h=[1,2,3].values().flatMap(function(v,i){seen.push(i);return i===1?[]:[v,v+10];});
assert(h.toArray().join(',')==='1,11,3,13' && seen.join(',')==='0,1,2','empty inner increments outer counter');
var token={}, log=[], source={next:function(){return {value:1,done:false};},return:function(){log.push('outer');return {};}};
h=Iterator.prototype.flatMap.call(source,function(){return {next:function(){throw token;},return:function(){log.push('inner');return {};}};});
try{h.next();throw new Error('inner step');}catch(e){assert(e===token,'inner step original');}
assert(log.join(',')==='outer' && h.next().done,'failed inner step excluded from Close');
"#,
    );
}

#[test]
fn terminal_short_circuit_closes_and_preserves_whole_result() {
    assert_helpers(
        r#"
var n=0,closed=0,token={}, source={next:function(){return {value:++n,done:false};},return:function(){closed++;return {}; }};
assert(Iterator.prototype.every.call(source,function(v){return v<2;})===false && n===2 && closed===1,'every short circuit');
n=0;assert(Iterator.prototype.some.call(source,function(v){return v===3;})===true && n===3 && closed===2,'some short circuit');
n=0;assert(Iterator.prototype.find.call(source,function(v){return v===2;})===2 && n===2 && closed===3,'find short circuit');
source.return=function(){throw token;};n=0;
try{Iterator.prototype.some.call(source,function(){return true;});throw new Error('close error');}catch(e){assert(e===token,'Normal loses to close Throw');}
assert([].values().every(function(){throw token;})===true && [].values().some(function(){throw token;})===false && [].values().find(function(){throw token;})===undefined,'empty terminals');
"#,
    );
}

#[test]
fn terminal_reduce_presence_counter_and_close() {
    assert_helpers(
        r#"
var indices=[];assert([1,2,3].values().reduce(function(a,v,i){indices.push(i);return a+v;})===6 && indices.join(',')==='1,2','no initial');
indices=[];assert([1,2].values().reduce(function(a,v,i){indices.push(i);return (a===undefined?0:a)+v;},undefined)===3 && indices.join(',')==='0,1','present undefined initial');
var token={}, closed=0;
try{Iterator.prototype.reduce.call({next:function(){return {value:3,done:false};},return:function(){closed++;return {}; }},function(){throw token;},0);throw new Error('callback');}catch(e){assert(e===token && closed===1,'reducer Throw closes');}
try{[].values().reduce(function(){});throw new Error('empty');}catch(e){assert(e instanceof TypeError,'empty absent initial');}
var seen=[];assert([2,4].values().forEach(function(v,i){'use strict';seen.push(v+i);assert(this===undefined,'callback this');})===undefined && seen.join(',')==='2,5','forEach');
"#,
    );
}

#[test]
fn concat_caches_open_methods_and_opens_only_on_demand() {
    assert_helpers(
        r#"
var log=[],one={get [Symbol.iterator](){log.push('get1');return function(){log.push('open1');var i=0;return {next:function(){log.push('next1');return i++===0?{value:1,done:false}:{done:true};},return:function(){log.push('close1');return {};}};};}},two={get [Symbol.iterator](){log.push('get2');return function(){log.push('open2');return [2].values();};}};
var h=Iterator.concat(one,two);assert(log.join(',')==='get1,get2','only methods acquired eagerly');
Object.defineProperty(one,Symbol.iterator,{value:function(){throw new Error('cached');}});
assert(h.next().value===1 && log.join(',')==='get1,get2,open1,next1','first opens only when needed');
assert(h.return().done && log.join(',')==='get1,get2,open1,next1,close1','only active source closes');
log=[];h=Iterator.concat(one,two);h.return();assert(log.join(',')==='get2','start return opens nothing');
assert(Iterator.concat([1,2],[],[3]).toArray().join(',')==='1,2,3','ordered lazy concat');
try{Iterator.concat('ab');throw new Error('String');}catch(e){assert(e instanceof TypeError,'concat rejects primitives');}
"#,
    );
}

#[test]
fn borrowed_helper_resume_and_terminal_realms() {
    assert_helpers(
        r#"
var realm=$262.createRealm(),other=realm.global, localProto=Object.getPrototypeOf([1].values().map(function(v){return v;}));
var next=localProto.next, close=localProto.return;
var source={i:0,next:function(){return this.i++===0?{value:9,done:false}:{done:true};}};
var h=other.Iterator.prototype.map.call(source,function(v){return v;});
assert(Object.getPrototypeOf(next.call(h))===other.Object.prototype,'active resume creation Realm');
assert(Object.getPrototypeOf(next.call(h))===other.Object.prototype,'exhaustion in creation Realm');
assert(Object.getPrototypeOf(next.call(h))===Object.prototype,'already-completed in called Realm');
h=other.Iterator.prototype.map.call({next:function(){return {value:1,done:false};},return:function(){return {}; }},function(v){return v;});
assert(Object.getPrototypeOf(close.call(h))===Object.prototype,'suspended-start return called Realm');
h=other.Iterator.prototype.map.call({next:function(){return 3;}},function(v){return v;});
try{next.call(h);throw new Error('protocol');}catch(e){assert(e instanceof other.TypeError && !(e instanceof TypeError),'active protocol error creation Realm');}
var a=other.Iterator.prototype.toArray.call([1,2].values());assert(Object.getPrototypeOf(a)===other.Array.prototype && a.join(',')==='1,2','terminal called Realm');
"#,
    );
}

#[test]
fn zip_modes_padding_and_whole_rows() {
    assert_helpers(
        r#"
var symbol=Symbol('zip'), token={}, closed=[], a={i:0,next:function(){return this.i++===0?{value:token,done:false}:{done:true};},return:function(){closed.push('a');return {};}},b={i:0,next:function(){return this.i++<2?{value:symbol,done:false}:{done:true};},return:function(){closed.push('b');return {};}};
var h=Iterator.zip([a,b]);var first=h.next();assert(!first.done && first.value[0]===token && first.value[1]===symbol,'whole shortest row');assert(h.next().done && closed.join(',')==='b','shortest removes completed before reverse close');
var paddingLog=[],padding={ [Symbol.iterator]:function(){paddingLog.push('open');return {next:function(){paddingLog.push('next');return {value:9n,done:false};},return:function(){paddingLog.push('close');return {};}};}};
h=Iterator.zip([[1],[2,3]],{mode:'longest',padding:padding});assert(paddingLog.join(',')==='open,next,next,close','padding stage consumes count then closes');assert(h.next().value.join(',')==='1,2','longest first');var row=h.next();assert(!row.done && row.value[0]===9n && row.value[1]===3,'longest retained padding');assert(h.next().done,'no final all-padding row');
paddingLog=[];h=Iterator.zip([],{mode:'longest',padding:padding});assert(paddingLog.join(',')==='open,close' && h.next().done,'empty zip still acquires and closes supplied padding');
var sentinel={},reads=0;var options={mode:'shortest',get padding(){reads++;throw sentinel;}};assert(Iterator.zip([],options).next().done && reads===0,'padding only longest');try{Iterator.zip([],{mode:{toString:function(){throw sentinel;}}});throw new Error('coerced mode');}catch(e){assert(e instanceof TypeError && e!==sentinel,'mode membership does not coerce');}
"#,
    );
}

#[test]
fn zip_acquisition_and_step_failure_close_order() {
    assert_helpers(
        r#"
var token={},log=[];
function inner(name){return {get next(){log.push(name+':next-get');return function(){return {value:1,done:false};};},return:function(){log.push(name+':return');throw {};}};}
var first=inner('a'),second=inner('b'),n=0,outer={ [Symbol.iterator]:new Proxy(function(){return this;},{apply:function(target,receiver,args){assert(receiver===outer && args.length===0,'outer raw receiver/argv');return receiver;}}),next:function(){return n++<2?{value:n===1?first:second,done:false}:{value:1,done:false};},return:function(){log.push('outer:return');throw {};}};
try{Iterator.zip(outer);throw new Error('primitive accepted');}catch(e){assert(e instanceof TypeError,'inner primitive rejected');}assert(log.join(',')==='a:next-get,b:next-get,b:return,a:return,outer:return','inner admission closes inners reverse then outer');
log=[];n=0;outer={ [Symbol.iterator]:function(){return this;},next:function(){if(n++===0)return {value:inner('a'),done:false};throw token;},return:function(){log.push('outer:return');return {};}};
try{Iterator.zip(outer);throw new Error('outer step');}catch(e){assert(e===token,'outer original Throw');}assert(log.join(',')==='a:next-get,a:return','failed outer step is not closed');
log=[];var a={next:function(){return {value:1,done:false};},return:function(){log.push('a');throw {};}},b={next:new Proxy(function(){return 3;},{}),return:function(){log.push('b');return {};}},c={next:function(){return {done:true};},return:function(){log.push('c');throw {};}};
var h=Iterator.zip([a,b,c]);try{h.next();throw new Error('primitive result');}catch(e){assert(e instanceof TypeError,'step protocol error');}assert(log.join(',')==='c,a' && h.next().done,'failing record removed, others reverse closed');
"#,
    );
}

#[test]
fn zip_strict_completion_probes_omit_value_and_defer_next_callability() {
    assert_helpers(
        r#"
var reads=0,closed=[],a={next:function(){return {done:true,get value(){reads++;throw {};}};}},b={next:function(){return {done:false,get value(){reads++;throw {};}};},return:function(){closed.push('b');return {};}},c={next:function(){return {done:true};},return:function(){closed.push('c');return {};}};
var h=Iterator.zip([a,b,c],{mode:'strict'});try{h.next();throw new Error('mismatch');}catch(e){assert(e instanceof TypeError,'strict mismatch');}assert(reads===0 && closed.join(',')==='c,b' && h.next().done,'probe omits value and closes all remaining');
h=Iterator.zip([{next:function(){return {done:true,get value(){throw {};}};}},{next:function(){return {done:true,get value(){throw {};}};}}],{mode:'strict'});assert(h.next().done,'equal empty strict probes omit value');
var nextGets=0;h=Iterator.zip([{get next(){nextGets++;return 1;}}]);assert(nextGets===1,'cached next callability deferred');try{h.next();throw new Error('cached noncallable');}catch(e){assert(e instanceof TypeError && nextGets===1,'one cached next Get');}
"#,
    );
}

#[test]
fn zip_keyed_descriptor_get_padding_and_null_prototype() {
    assert_helpers(
        r#"
var log=[],symbol=Symbol('key'), token={},values={skip:[8],a:[token],b:[2,3]};Object.defineProperty(values,'skip',{enumerable:false});values[symbol]=[4];values.absent=undefined;
var input=new Proxy(values,{ownKeys:function(){log.push('keys');return ['skip','a','gone','absent','b',symbol];},getOwnPropertyDescriptor:function(target,key){log.push('desc:'+String(key));return Object.getOwnPropertyDescriptor(target,key);},get:function(target,key,receiver){log.push('get:'+String(key));return Reflect.get(target,key,receiver);}});
var pads={get a(){log.push('pad:a');return 7;},get b(){log.push('pad:b');return 8;},get [symbol](){log.push('pad:s');return 9;}};
var h=Iterator.zipKeyed(input,{mode:'longest',padding:pads});assert(log.join(',')==='keys,desc:skip,desc:a,get:a,desc:gone,desc:absent,get:absent,desc:b,get:b,desc:Symbol(key),get:Symbol(key),pad:a,pad:b,pad:s','OwnKeys descriptor/Get then padding order');
var row=h.next().value;assert(Object.getPrototypeOf(row)===null && row.a===token && row.b===2 && row[symbol]===4 && !('skip' in row) && !('absent' in row),'null-prototype keyed whole row');row=h.next().value;assert(row.a===7 && row.b===3 && row[symbol]===9 && h.next().done,'keyed longest padding');
var error={},closed=[],a={next:function(){return {done:true};},return:function(){closed.push('a');return {};}},b={next:function(){return {done:true};},return:function(){closed.push('b');throw {};}};
try{Iterator.zipKeyed({a:a,b:b},{mode:'longest',padding:{get a(){throw error;}}});throw new Error('padding Throw');}catch(e){assert(e===error && closed.join(',')==='b,a','padding Throw reverse closes original iterators');}
"#,
    );
}

#[test]
fn zip_return_close_priority_reentrancy_and_realms() {
    assert_helpers(
        r#"
var firstError={},secondError={},closed=[],h;
function source(name,error){return {next:function(){return {value:1,done:false};},return:function(){closed.push(name);if(error)throw error;return {};}};}
h=Iterator.zip([source('a',secondError),source('b',firstError)]);try{h.return();throw new Error('close');}catch(e){assert(e===firstError,'first reverse close Throw wins');}assert(closed.join(',')==='b,a' && h.next().done,'all close even after Throw');
var observed=false;h=Iterator.zip([{next:function(){return {value:1,done:false};},return:function(){observed=h.next().done;return {};}}]);assert(h.return().done && observed,'suspended-start return marks completed before close');
var running=false;h=Iterator.zip([{next:function(){return {value:1,done:false};},return:function(){try{h.next();}catch(e){running=e instanceof TypeError;}return {};}}]);h.next();assert(h.return().done && running,'resumed return protects executing state');
var realm=$262.createRealm(),other=realm.global,proto=Object.getPrototypeOf(Iterator.zip([])),next=proto.next,close=proto.return;
h=other.Iterator.zip([[1],[2]]);var result=next.call(h);assert(Object.getPrototypeOf(result)===other.Object.prototype && Object.getPrototypeOf(result.value)===other.Array.prototype,'active zip result and row creation Realm');assert(Object.getPrototypeOf(next.call(h))===other.Object.prototype,'exhaustion creation Realm');assert(Object.getPrototypeOf(next.call(h))===Object.prototype,'completed resume called Realm');
h=other.Iterator.zip([[1]]);assert(Object.getPrototypeOf(close.call(h))===Object.prototype,'unstarted return called Realm');
h=other.Iterator.zip([{next:function(){return 3;}}]);try{next.call(h);throw new Error('protocol');}catch(e){assert(e instanceof other.TypeError && !(e instanceof TypeError),'active protocol error creation Realm');}
"#,
    );
}
