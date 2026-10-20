function check(condition,label){if(!condition)throw label;}
var whole={tag:'whole'};whole.self=whole;
whole[Symbol.toPrimitive]=function(){throw 'unexpected-conversion';};
async function run(){
  var nextGets=0, steps=0, closed=0, readers=[], iterator;
  var record={get next(){nextGets++;return function(){steps++;return {value:steps,done:steps>2};};},return:function(){closed++;return {};}};
  var iterable={[Symbol.iterator]:function(){return record;}};
  async function* sync(){for(const value of await(yield 'head')){readers.push(()=>value);await Promise.resolve(0);yield value;}}
  iterator=sync();check((await iterator.next()).value==='head','complete-sync-head');
  check((await iterator.next(iterable)).value===1,'first-sync-value');
  Object.defineProperty(record,'next',{value:function(){throw 'uncached-next';},configurable:true});gc();
  check((await iterator.next()).value===2,'cached-original-next');
  check((await iterator.next()).done&&nextGets===1&&steps===3&&closed===0,'exhaustion-does-not-close');
  check(readers[0]()===1&&readers[1]()===2,'distinct-original-per-key-cells');
  var log=[], count=0;
  var asyncRecord={next:function(){log.push('next');return Promise.resolve({value:++count,done:count>2});},return:function(){log.push('return');return Promise.resolve({});}};
  var both={[Symbol.asyncIterator]:function(){log.push('async');return asyncRecord;},[Symbol.iterator]:function(){throw 'unexpected-sync-method';}};
  async function* awaited(){for await(const value of await(yield 'awaited-head')){await Promise.resolve(0);yield value;}}
  iterator=awaited();await iterator.next();
  check((await iterator.next(both)).value===1,'async-method-preferred');gc();
  check((await iterator.next()).value===2,'awaited-next-retained');
  check((await iterator.next()).done&&log.join(',')==='async,next,next,next','async-exhaustion-no-return');
  var adopted=0, wrapped={[Symbol.iterator]:function(){var sent=false;return {next:function(){if(sent)return {done:true};sent=true;return {done:false,value:{then:function(resolve){adopted++;resolve(whole);}}};}};}};
  async function* fallback(){for await(const value of wrapped){yield value;}}
  iterator=fallback();check((await iterator.next()).value===whole&&adopted===1,'async-from-sync-value-adoption');
  check((await iterator.next()).done,'wrapped-exhaustion');
  var headReader;
  function capture(read){headReader=read;return 'tdz-head';}
  async function* tdz(){for(const value of await(yield capture(()=>value))){yield ()=>value;}}
  iterator=tdz();await iterator.next();
  try{headReader();throw 'missing-head-tdz';}catch(error){check(error instanceof ReferenceError,'head-original-tdz');}
  var perKey=(await iterator.next([whole])).value;gc();check(perKey()===whole,'per-key-publication');
  try{headReader();throw 'initialized-head-tdz';}catch(error){check(error instanceof ReferenceError,'head-and-iteration-records-distinct');}
  await iterator.next();gc();check(perKey()===whole,'escaping-cell-after-retirement');
  closed=0;
  var broken={[Symbol.iterator]:function(){return {next:function(){throw whole;},return:function(){closed++;return {};}};}};
  async function* stepFailure(){try{for(const value of broken){yield value;}}catch(error){check(error===whole,'step-whole-throw');yield 'caught';}}
  iterator=stepFailure();check((await iterator.next()).value==='caught'&&closed===0,'step-failure-marks-done');await iterator.next();
  async function* nullish(){try{for(const value of await(yield 'null-head')){throw 'entered-null-body';}}catch(error){check(error instanceof TypeError,'null-acquisition-type-error');yield 'null-caught';}}
  iterator=nullish();await iterator.next();check((await iterator.next(null)).value==='null-caught','acquisition-before-own-close');await iterator.next();
}
run().then(function(){print('mixed-async-generator-for-of-iterators:ok');},function(error){print(error);throw error;});
