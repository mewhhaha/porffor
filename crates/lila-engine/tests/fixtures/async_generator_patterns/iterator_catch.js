function check(value,label){if(!value)throw label;}
var whole={tag:'whole'},events=[];
function outer(){var count=0;return {[Symbol.iterator](){return this;},next(){events.push('outer-next');return count++?{done:true}:{value:2,done:false};},return(){events.push('outer-close');return {};}};}
function inner(){return {[Symbol.iterator](){return this;},next(){events.push('inner-next');return {value:undefined,done:false};},return(){events.push('inner-close');return {};}};}
function* values(list,input){for(const item of list){try{throw input;}catch([received=yield item]){yield received;}}}
function generators(){
  var iterator=values(outer(),inner());check(iterator.next().value===2,'default-suspends-inside-current-iteration');gc();
  check(iterator.next(whole).value===whole&&events.join(',')==='outer-next,inner-next,inner-close','normal-catch-initialization-closes-inner-before-body');
  check(iterator.next().done&&events.join(',')==='outer-next,inner-next,inner-close,outer-next','normal-outer-exhaustion');
  events=[];iterator=values(outer(),inner());iterator.next();gc();
  var result=iterator.return(whole);
  check(result.done&&result.value===whole&&events.join(',')==='outer-next,inner-next,inner-close,outer-close','injected-return-inner-before-outer-close');
  events=[];iterator=values(outer(),inner());iterator.next();
  try{iterator.throw(whole);throw 'lost-whole';}catch(error){check(error===whole,'injected-whole-throw');}
  check(events.join(',')==='outer-next,inner-next,inner-close,outer-close','injected-throw-close-order');
}
async function asynchronous(){
  async function sync(list,input){for(const item of list){try{throw input;}catch([received=await item]){return received;}}}
  async function awaited(list,input){for await(const item of list){try{throw input;}catch([received=await item]){return received;}}}
  for(const operation of [sync,awaited]){
    events=[];check(await operation(outer(),inner())===2,'awaited-default-publishes-catch-binding');
    check(events.join(',')==='outer-next,inner-next,inner-close,outer-close','normal-return-close-order');
  }
  async function rejected(list,input){for(const item of list){try{throw input;}catch([received=await Promise.reject(whole)]){throw 'unexpected-body';}}}
  events=[];try{await rejected(outer(),inner());throw 'lost-rejection';}catch(error){check(error===whole,'whole-default-rejection');}
  check(events.join(',')==='outer-next,inner-next,inner-close,outer-close','rejection-close-order');
}
generators();asynchronous().then(()=>print('iterator-catch:ok'),error=>{print(error);throw error;});
262;
