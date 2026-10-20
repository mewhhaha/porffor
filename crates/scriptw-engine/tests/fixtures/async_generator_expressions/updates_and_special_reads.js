function check(condition,label){if(!condition)throw label;}
var whole={tag:'whole'};whole.self=whole;
async function run(){
  var events=[],base={marker:11},alternate={marker:19},coercions=0,gets=0,sets=0;
  var old={valueOf:function(){events.push('numeric');return 5;}},target={};
  Object.defineProperty(target,'selected',{configurable:true,get:function(){gets++;events.push('get');return old;},set:function(value){sets++;events.push('set');this.received=value;}});
  var key={[Symbol.toPrimitive]:function(){coercions++;events.push('key');return 'selected';}};
  async function* ordinary(){const result=(yield target)[await(yield 'key')]++;yield result;}
  var iterator=ordinary();check((await iterator.next()).value===target,'update-retains-original-base');
  check((await iterator.next(target)).value==='key'&&gets===0,'update-no-get-before-computed-reference');
  gc();check((await iterator.next(key)).value===5&&target.received===6&&gets===1&&sets===1&&coercions===1,'post-update-original-reference-and-result');
  check(events.join(',')==='key,get,numeric,set','key-get-tonumeric-put-order');await iterator.next();

  class Parent{}
  Object.defineProperty(Parent.prototype,'chosen',{configurable:true,get:function(){events.push('super-get');return this.marker;},set:function(value){events.push('super-set');this.received=value;}});
  class C extends Parent{
    #value=23;
    get #method(){gets++;return function(value){return [this,value];};}
    async* values(){
      const branded=#value in(await(yield this));
      const superValue=super[await(yield 'super-key')];
      const privateOld=(await(yield this)).#value++;
      const superOld=super[await(yield 'super-update')]++;
      yield [branded,superValue,privateOld,superOld,this.#value,this.received];
    }
    async* brands(){try{yield #value in(await(yield 'brand-target'));}catch(error){check(error instanceof TypeError,'private-in-original-primitive-rejection');yield 'caught';}}
    async* optional(){const value=(yield this)?.#method(await(yield 'private-argument'));yield value;}
    async* grouped(){const value=((yield this)?.#method)(await(yield 'grouped-private-argument'));yield value;}
    async* skipped(){const value=(yield null)?.#method(await(yield 'unreached-private-argument'));yield value;}
    async* invalid(){try{(yield {})?.#method(await(yield 'unreached-brand-argument'));}catch(error){check(error instanceof TypeError,'optional-private-original-brand-check');yield 'caught-brand';}}
    async* deleted(){try{delete super[await(yield 'super-delete')];}catch(error){check(error instanceof ReferenceError,'original-super-delete-reference-error');yield 'caught-delete';}}
  }
  var receiver=new C();receiver.marker=31;events=[];iterator=receiver.values();
  check((await iterator.next()).value===receiver,'private-in-target-prefix');
  check((await iterator.next(receiver)).value==='super-key','private-in-before-super-key');
  check((await iterator.next('chosen')).value===receiver&&events.join(',')==='super-get','held-super-receiver-Get');
  gc();check((await iterator.next(receiver)).value==='super-update','original-private-update-after-mixed-target');
  receiver.marker=37;var result=(await iterator.next('chosen')).value;
  check(result[0]===true&&result[1]===31&&result[2]===23&&result[3]===37&&result[4]===24&&result[5]===38,'original-private-and-super-numeric-operations');
  check(events.join(',')==='super-get,super-get,super-set','super-original-receiver-one-read-one-write');await iterator.next();
  iterator=receiver.brands();await iterator.next();check((await iterator.next(0)).value==='caught','private-in-check-after-completed-await');await iterator.next();
  gets=0;iterator=receiver.optional();await iterator.next();check((await iterator.next(receiver)).value==='private-argument'&&gets===1,'optional-private-get-before-selected-argument');
  gc();result=(await iterator.next(whole)).value;check(result[0]===receiver&&result[1]===whole&&gets===1,'optional-private-original-reference-receiver');await iterator.next();
  iterator=receiver.grouped();await iterator.next();check((await iterator.next(receiver)).value==='grouped-private-argument'&&gets===2,'grouped-private-terminal-reference');
  result=(await iterator.next(whole)).value;check(result[0]===receiver&&result[1]===whole&&gets===2,'grouped-private-receiver-survives-await');await iterator.next();
  iterator=receiver.skipped();await iterator.next();check((await iterator.next(null)).value===undefined&&gets===2,'private-nullish-skips-brand-and-argument');await iterator.next();
  iterator=receiver.invalid();await iterator.next();check((await iterator.next({})).value==='caught-brand'&&gets===2,'private-brand-failure-before-arguments');await iterator.next();
  var rawSuperKey={[Symbol.toPrimitive]:function(){throw 'super-delete-coerced-key';}};
  iterator=receiver.deleted();check((await iterator.next()).value==='super-delete','super-delete-complete-key-prefix');
  check((await iterator.next(rawSuperKey)).value==='caught-delete','super-delete-original-uncoerced-reference-error');await iterator.next();

  async function* bigint(){const value=++(await(yield target)).selected;yield value;}
  Object.defineProperty(target,'selected',{configurable:true,writable:true,value:41n});
  iterator=bigint();await iterator.next();check((await iterator.next(target)).value===42n&&target.selected===42n,'original-dynamic-bigint-update');await iterator.next();

  var resolve,promise=new Promise(function(done){resolve=done;}),queueTarget={selected:43};
  async function* queued(){try{const result=(yield queueTarget)[await(yield 'queue-key')]++;yield result;}finally{await Promise.resolve();yield 'finally';}}
  iterator=queued();await iterator.next();await iterator.next(queueTarget);var next=iterator.next(promise),returned=iterator.return(whole);gc();resolve('selected');
  check((await next).value===43&&queueTarget.selected===44,'update-finishes-before-queued-return');check((await returned).value==='finally','queued-return-yielding-finalizer');
  result=await iterator.next();check(result.done&&result.value===whole,'whole-return-after-update-finalizer');
}
run().then(function(){print('mixed-async-generator-updates-special-reads:ok');},function(error){print(error);throw error;});
