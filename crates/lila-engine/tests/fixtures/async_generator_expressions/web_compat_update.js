function check(condition,label){if(!condition)throw label;}
var whole={tag:'whole'};
async function run(){
  var calls=0,received,record={call:function(value){calls++;received=value;check(this===record,'original-annex-b-call-receiver');return 1;}};
  async function* updating(){try{record.call(await(yield 'argument'))++;}catch(error){check(error instanceof ReferenceError,'original-update-reference-error-after-call');yield 'caught';}}
  var iterator=updating();check((await iterator.next()).value==='argument'&&calls===0,'annex-b-await-prefix-before-call');
  gc();check((await iterator.next(whole)).value==='caught'&&calls===1&&received===whole,'whole-original-argument-and-once-only-call');await iterator.next();
  record.call=function(){throw whole;};
  async function* throwing(){try{record.call(await(yield 'throw'))++;}catch(error){check(error===whole,'original-call-error-precedes-update-reference-error');yield 'caught-whole';}}
  iterator=throwing();await iterator.next();check((await iterator.next(1)).value==='caught-whole','actual-call-abrupt-completion');await iterator.next();
}
run().then(function(){print('mixed-async-generator-annex-b-update:ok');},function(error){print(error);throw error;});
