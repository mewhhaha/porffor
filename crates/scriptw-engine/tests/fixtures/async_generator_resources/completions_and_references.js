function check(condition,label){if(!condition)throw label;}
var whole={tag:'whole'},low={tag:'low'},high={tag:'high'},selected='outer';whole.self=whole;
async function run(){
  var events=[],record={selected:2},excluded={selected:false};record[Symbol.unscopables]=excluded;
  function disposable(name,error){return {[Symbol.asyncDispose]:function(){events.push(name);return error?Promise.reject(error):Promise.resolve();}};}
  async function* retained(){await using resource=await(yield 'resource-head');with(record){selected+=await(yield 'rhs');yield selected;}}
  var iterator=retained();await iterator.next();check((await iterator.next(disposable('held'))).value==='rhs','resource-then-selected-reference');
  excluded.selected=true;gc();check((await iterator.next(4)).value==='outer'&&record.selected===6,'reference-retained-through-resource-scope');
  check((await iterator.next()).done&&events.join(',')==='held','original-record-cleanup');
  events=[];
  async function* finishing(){await using resource=disposable('dispose');try{yield 'body';}finally{await Promise.resolve();gc();check(events.length===0,'scope-stays-live-through-finally');yield 'finally';}}
  iterator=finishing();await iterator.next();var returned=iterator.return(whole),queued=iterator.next();
  // A queued next may advance cleanup before the caller's promise reaction.
  // Observe the live scope inside finally; keep the queued Return identity check.
  check((await returned).value==='finally','queued-return-yields-finally');
  var terminal=await queued;check(terminal.done&&terminal.value===whole&&events.join(',')==='dispose','whole-queued-return-after-disposal');
  events=[];iterator=finishing();await iterator.next();check((await iterator.throw(whole)).value==='finally','injected-throw-yielding-finalizer');
  try{await iterator.next();throw 'missing-original-throw';}catch(error){check(error===whole&&events.join(',')==='dispose','whole-throw-after-resource-cleanup');}
  events=[];
  async function* suppressed(){await using first=disposable('first',low),second=disposable('second',high);yield 'body';throw whole;}
  iterator=suppressed();await iterator.next();try{await iterator.next();throw 'missing-suppression';}catch(error){
    check(error instanceof SuppressedError&&error.error===low,'last-disposer-is-outer-error');
    check(error.suppressed instanceof SuppressedError&&error.suppressed.error===high&&error.suppressed.suppressed===whole,'whole-nested-suppression');
    check(events.join(',')==='second,first','both-original-entries-disposed');
  }
  events=[];var closes=0;
  var source={[Symbol.iterator]:function(){var sent=false;return {next:function(){if(sent)return {done:true};sent=true;return {done:false,value:1};},return:function(){closes++;return {};}};}};
  async function* withinIterator(){for await(const value of source){await using resource=await(yield value);yield resource;}}
  iterator=withinIterator();await iterator.next();await iterator.next(disposable('per-key'));terminal=await iterator.return(whole);
  check(terminal.done&&terminal.value===whole&&events.join(',')==='per-key'&&closes===1,'resource-disposal-before-original-iterator-close');
  events=[];
  async function* targetError(){using resource={[Symbol.dispose]:function(){events.push('sync');throw low;}};yield 'body';}
  iterator=targetError();await iterator.next();try{await iterator.return(whole);throw 'missing-dispose-error';}catch(error){check(error===low&&events.join(',')==='sync','sync-disposal-error-overrides-return');}
  events=[];
  async function* caseFailure(){try{switch(yield 'case'){case 1:{await using first=disposable('first'),second=await(yield 'later');yield 'unreached';}}}catch(error){check(error===whole,'caseblock-whole-rejection');yield 'caught-case';}}
  iterator=caseFailure();await iterator.next();check((await iterator.next(1)).value==='later','first-case-entry-registers');
  check((await iterator.next(Promise.reject(whole))).value==='caught-case'&&events.join(',')==='first','caseblock-earlier-entry-disposes-on-later-rejection');await iterator.next();
  events=[];
  async function* caseBreak(){switch(yield 'case-break'){case 1:{using first={[Symbol.dispose]:function(){events.push('break-dispose');}};yield 'inside-case';break;}case 2:{await using unreached=disposable('unreached');}}yield 'outside-case';}
  iterator=caseBreak();await iterator.next();await iterator.next(1);
  check((await iterator.next()).value==='outside-case'&&events.join(',')==='break-dispose','break-disposes-before-leaving-caseblock');await iterator.next();
  events=[];
  async function* headRejection(){try{for(await using held=disposable('head');await(yield 'head-test');yield 'update'){yield 'body';}}catch(error){check(error===whole,'classic-test-whole-rejection');yield 'caught-head';}}
  iterator=headRejection();await iterator.next();check((await iterator.next(Promise.reject(whole))).value==='caught-head'&&events.join(',')==='head','classic-test-rejection-disposes-head');await iterator.next();
}
run().then(function(){print('mixed-async-generator-resource-completions:ok');},function(error){print(error);throw error;});
