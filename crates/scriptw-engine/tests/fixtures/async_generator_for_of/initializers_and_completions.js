function check(condition,label){if(!condition)throw label;}
var whole={tag:'whole'};whole.self=whole;
var selected='outer';
async function run(){
  var log=[], put=[], target={set key(value){log.push('put');put.push(value);}}, other={};
  function input(value){return {[Symbol.iterator]:function(){var sent=false;return {next:function(){log.push('next');if(sent)return {done:true};sent=true;return {done:false,value:value};},return:function(){log.push('close');return {};}};}};}
  async function* member(){for((yield 'target')[await(yield 'key')] of await(yield 'input')){yield 'body';}}
  var iterator=member();check((await iterator.next()).value==='input','iterable-before-target');
  check((await iterator.next(input(whole))).value==='target'&&log.join(',')==='next','step-before-reference');gc();
  check((await iterator.next(target)).value==='key','raw-base-retained');
  var raw={toString:function(){log.push('key');return 'key';}};
  check((await iterator.next(raw)).value==='body'&&put[0]===whole&&log.join(',')==='next,key,put','raw-key-normalized-at-put');
  await iterator.next();
  log=[];
  function inner(){return {[Symbol.iterator]:function(){return {next:function(){log.push('inner-next');return {done:false,value:undefined};},return:function(){log.push('inner-close');return {};}};}};}
  async function* pattern(){for await(const [value=await(yield 'default')] of input(inner())){yield value;}}
  iterator=pattern();check((await iterator.next()).value==='default','complete-per-key-array-default');gc();
  check((await iterator.next(whole)).value===whole&&log.join(',')==='next,inner-next,inner-close','inner-closes-before-body');
  check((await iterator.return(whole)).value===whole&&log.join(',')==='next,inner-next,inner-close,close','outer-closes-on-return');
  var scope={selected:1}, excluded={selected:false};scope[Symbol.unscopables]=excluded;
  async function* reference(){with(scope){for(selected of await(yield 'reference-input')){selected+=await(yield 'rhs');yield selected;}}}
  iterator=reference();await iterator.next();check((await iterator.next([4])).value==='rhs'&&scope.selected===4,'original-per-key-put');
  excluded.selected=true;gc();check((await iterator.next(3)).value==='outer'&&scope.selected===7,'compound-held-object-record');await iterator.next();
  var closed=0;
  var source={[Symbol.iterator]:function(){return {next:function(){return {done:false,value:[undefined]};},return:function(){closed++;return {};}};}};
  async function* reject(){try{for(const [value=await(yield 'reject-default')] of source){throw 'entered-rejected-body';}}catch(error){check(error===whole,'default-whole-rejection');yield 'caught';}}
  iterator=reject();await iterator.next();check((await iterator.next(Promise.reject(whole))).value==='caught'&&closed===1,'rejected-initialization-closes-outer');await iterator.next();
  closed=0;var final=[];
  async function* finish(){for await(const value of source){try{yield 'body';}finally{await Promise.resolve(0);gc();final.push(value);yield 'finally';}}}
  iterator=finish();await iterator.next();var returning=iterator.return(whole),queued=iterator.next();
  check((await returning).value==='finally','queued-return-enters-yielding-finally');
  var terminal=await queued;check(terminal.done&&terminal.value===whole&&closed===1&&final.length===1,'whole-return-through-awaited-close');
  closed=0;iterator=finish();await iterator.next();check((await iterator.throw(whole)).value==='finally','injected-throw-finalizer');
  try{await iterator.next();throw 'missing-throw';}catch(error){check(error===whole&&closed===1,'whole-throw-through-close');}
  var closeError={tag:'close-error'};
  var hostile={[Symbol.iterator]:function(){return {next:function(){return {done:false,value:1};},return:function(){throw closeError;}};}};
  async function* hostileLoop(){for(const value of hostile){yield value;}}
  iterator=hostileLoop();await iterator.next();try{await iterator.throw(whole);throw 'missing-original-throw';}catch(error){check(error===whole,'throw-wins-close-error');}
  iterator=hostileLoop();await iterator.next();try{await iterator.return(whole);throw 'missing-close-error';}catch(error){check(error===closeError,'return-observes-close-error');}
  var events=[],headReader,bodyReader,startDisposal,releaseDisposal;
  var started=new Promise(function(resolve){startDisposal=resolve;});
  var resource={[Symbol.asyncDispose]:function(){events.push('dispose');startDisposal();return new Promise(function(resolve){releaseDisposal=resolve;});}};
  function resources(){return {[Symbol.iterator]:function(){var sent=false;return {next:function(){events.push('next');if(sent)return {done:true};sent=true;return {done:false,value:resource};},return:function(){events.push('close');return {};}};}};}
  async function* resourceHead(){for await(await using held of (headReader=function(){return held;},resources())){bodyReader=function(){return held;};yield held;}}
  iterator=resourceHead();check((await iterator.next()).value===resource,'resource-head-initializes-original-per-key-cell');
  try{headReader();throw 'initialized-head-tdz';}catch(error){check(error instanceof ReferenceError,'resource-head-keeps-distinct-tdz-record');}
  var pending=iterator.next();await started;gc();
  check(bodyReader()===resource&&events.join(',')==='next,dispose','captured-resource-live-during-disposer-await');
  try{headReader();throw 'initialized-head-tdz-during-disposal';}catch(error){check(error instanceof ReferenceError,'head-tdz-survives-disposer-await');}
  releaseDisposal();check((await pending).done&&events.join(',')==='next,dispose,next','dispose-before-next');
  events=[];resource={[Symbol.asyncDispose]:function(){events.push('dispose');return Promise.resolve();}};
  iterator=resourceHead();await iterator.next();terminal=await iterator.return(whole);
  check(terminal.done&&terminal.value===whole&&events.join(',')==='next,dispose,close','resource-head-disposes-before-abrupt-iterator-close');
  events=[];var firstResource={[Symbol.dispose]:function(){events.push('first');}},secondResource={[Symbol.dispose]:function(){events.push('second');}};
  var syncResources={[Symbol.iterator]:function(){var index=0;return {next:function(){events.push('next');return index<2?{done:false,value:[firstResource,secondResource][index++]}:{done:true};}};}};
  async function* continueResource(){for(using held of syncResources){yield held;continue;}}
  iterator=continueResource();await iterator.next();check((await iterator.next()).value===secondResource&&events.join(',')==='next,first,next','continue-disposes-before-advance');
  check((await iterator.next()).done&&events.join(',')==='next,first,next,second,next','each-key-owns-one-capability');
  var superWrites=[];
  class SuperParent{set chosen(next){superWrites.push([this,next]);}}
  class SuperChild extends SuperParent{async* values(input){for await(super[await(yield 'super-key')] of input){yield this;}}}
  var receiver=new SuperChild();log=[];iterator=receiver.values(input(whole));
  check((await iterator.next()).value==='super-key'&&log.join(',')==='next','super-per-key-reference-after-next');gc();
  check((await iterator.next('chosen')).value===receiver&&superWrites.length===1&&superWrites[0][0]===receiver&&superWrites[0][1]===whole,'super-per-key-write-retains-incoming-and-this');
  check((await iterator.return(whole)).value===whole&&log.join(',')==='next,close','super-initializer-abrupt-body-closes-original-iterator');
}
run().then(function(){print('mixed-async-generator-for-of-initializers:ok');},function(error){print(error);throw error;});
