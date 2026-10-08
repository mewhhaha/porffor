function check(condition,label){if(!condition)throw label;}
var whole={tag:'whole'};whole.self=whole;
async function run(){
  var events=[],read;
  function resource(name,asynchronous){var value={};Object.defineProperty(value,asynchronous?Symbol.asyncDispose:Symbol.dispose,{get:function(){events.push('get:'+name);return function(){check(this===value,'held-resource-receiver');events.push('dispose:'+name);return asynchronous?Promise.resolve():undefined;};}});return value;}
  async function* acquiring(){yield 'prefix';using first=await(yield 'first'),second=await(yield 'second');await using third=await(yield 'third'),empty=null;read=()=>first;yield third;}
  var iterator=acquiring();check((await iterator.next()).value==='prefix','statements-before-declaration');
  check((await iterator.next()).value==='first','first-staged-initializer');gc();
  var first=resource('first',false),second=resource('second',false),third=resource('third',true);
  check((await iterator.next(first)).value==='second'&&events.join(',')==='get:first','first-registers-before-second-initializer');
  check((await iterator.next(second)).value==='third'&&events.join(',')==='get:first,get:second','same-capability-keeps-earlier-entry-live');
  check((await iterator.next(third)).value===third&&read()===first,'third-registration-and-captured-binding');gc();
  check((await iterator.next()).done&&events.join(',')==='get:first,get:second,get:third,dispose:third,dispose:second,dispose:first','mixed-declaration-one-reverse-walk');
  check(read()===first,'resource-binding-retained-after-disposal');
  events=[];
  async function* failure(){try{using earlier=resource('earlier',false),later=await(yield 'later');throw 'entered-failed-suffix';}catch(error){check(error===whole,'whole-initializer-rejection');yield 'caught';}}
  iterator=failure();await iterator.next();check((await iterator.next(Promise.reject(whole))).value==='caught'&&events.join(',')==='get:earlier,dispose:earlier','registered-before-later-rejection');await iterator.next();
  events=[];
  async function* invalid(){try{await using earlier=resource('async-earlier',true),later=await(yield 'invalid');}catch(error){check(error===whole,'whole-get-method-error');yield 'caught-method';}}
  iterator=invalid();await iterator.next();var broken={get [Symbol.asyncDispose](){events.push('broken-get');throw whole;}};
  check((await iterator.next(broken)).value==='caught-method'&&events.join(',')==='get:async-earlier,broken-get,dispose:async-earlier','get-method-before-binding-initialization');await iterator.next();
  events=[];
  async function* nested(){using outer=resource('outer',false);{await using inner=await(yield 'inner'),tail=resource('tail',true);yield ()=>inner;}yield outer;}
  iterator=nested();await iterator.next();var inner=resource('inner',true);read=(await iterator.next(inner)).value;gc();check(read()===inner,'nested-record-capture');
  check((await iterator.next()).value!==undefined&&events.join(',')==='get:outer,get:inner,get:tail,dispose:tail,dispose:inner','inner-disposal-before-outer-suffix');
  await iterator.next();check(events[events.length-1]==='dispose:outer','outer-disposes-at-own-scope-end');
  events=[];
  async function* nulls(){await using first=null,second=undefined;Promise.resolve().then(function(){events.push('tick');});yield 'null-body';}
  iterator=nulls();check((await iterator.next()).value==='null-body','null-resources-body');check((await iterator.next()).done&&events.join(',')==='tick','one-null-capability-finalizer');
  events=[];
  async function* classic(){for(await using held=await(yield 'classic-head'),empty=null;await(yield 'classic-test');yield 'classic-update'){read=()=>held;yield 'classic-body';continue;}}
  iterator=classic();check((await iterator.next()).value==='classic-head','complete-classic-resource-head');
  first=resource('classic',true);check((await iterator.next(first)).value==='classic-test','resource-registers-before-test');
  check((await iterator.next(true)).value==='classic-body','classic-first-body');gc();check(read()===first,'original-classic-head-cell');
  check((await iterator.next()).value==='classic-update'&&events.join(',')==='get:classic','continue-retains-whole-loop-capability');
  check((await iterator.next()).value==='classic-test'&&events.join(',')==='get:classic','update-retains-resource');
  check((await iterator.next(false)).done&&events.join(',')==='get:classic,dispose:classic','false-test-disposes-once-after-whole-loop');
  events=[];
  async function* cases(){switch(yield 'discriminant'){case await(yield 'selector'):{using first=await(yield 'case-first');read=()=>first;yield first;}case 2:{await using second=await(yield 'case-second');yield second;}default:yield 'tail';}}
  iterator=cases();check((await iterator.next()).value==='discriminant','case-resource-discriminant');
  check((await iterator.next(1)).value==='selector','lazy-selector-before-registration');
  check((await iterator.next(1)).value==='case-first','selected-case-initializer');
  first=resource('case-first',false);second=resource('case-second',true);
  check((await iterator.next(first)).value===first,'first-case-resource');
  check((await iterator.next()).value==='case-second'&&events.join(',')==='get:case-first,dispose:case-first','fallthrough-closes-first-block');
  check((await iterator.next(second)).value===second&&read()===first,'captured-first-block-record-survives-fallthrough');gc();
  check((await iterator.next()).value==='tail'&&events.join(',')==='get:case-first,dispose:case-first,get:case-second,dispose:case-second','default-fallthrough-follows-second-block-disposal');
  check((await iterator.next()).done&&events.join(',')==='get:case-first,dispose:case-first,get:case-second,dispose:case-second','each-clause-block-disposes-once');
  events=[];
  async function* unmatched(){switch(yield 'no-match'){case 1:{await using absent=resource('absent',true);yield absent;}}}
  iterator=unmatched();await iterator.next();check((await iterator.next(2)).done&&events.length===0,'unmatched-case-does-not-register');
}
run().then(function(){print('mixed-async-generator-resource-scopes:ok');},function(error){print(error);throw error;});
