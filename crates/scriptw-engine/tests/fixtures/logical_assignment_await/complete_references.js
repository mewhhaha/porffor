function check(condition,label){if(!condition)throw label;}
var whole={marker:91};whole[Symbol.toPrimitive]=function(){throw 'converted-whole';};
async function run(){
  var events=[],slot=0,receiver;
  var original={get chosen(){check(this===receiver,'logical-original-get-receiver');events.push('get');return slot;},set chosen(value){check(this===receiver,'logical-original-put-receiver');events.push('put');slot=value;}};
  receiver=new Proxy(original,{});
  var rawKey={[Symbol.toPrimitive]:function(){events.push('key');return 'chosen';}};
  var release;var waiting=new Promise(function(resolve){release=resolve;});
  async function choose(target,key,rhs){return (await target)[await key] ||= await rhs;}
  var completion=choose(receiver,rawKey,waiting);await 0;await 0;await 0;
  check(events.join(',')==='key,get','awaited-left-completes-before-get-and-rhs');
  rawKey[Symbol.toPrimitive]=function(){throw 'renormalized-key';};gc();release(whole);
  check(await completion===whole&&slot===whole&&events.join(',')==='key,get,put','logical-original-reference-survives-all-three-awaits');
  var adopted=0,skipped={get then(){adopted++;throw whole;}};
  check(await choose(receiver,'chosen',skipped)===whole&&adopted===0,'selected-old-value-skips-rhs-adoption');
  var gets=0;
  class Box{#value=0;async choose(target,rhs){return (await target).#value ||= await rhs;}read(){return this.#value;}}
  var box=new Box();
  check(await box.choose(box,whole)===whole&&box.read()===whole,'private-original-target-await-before-get-and-put');
  check(await box.choose(box,skipped)===whole&&adopted===0,'private-skipped-arm-never-adopts');
  var rejected={get then(){gets++;throw 'unexpected-private-rhs';}};
  try{await box.choose({},rejected);throw 'missing-private-brand-error';}catch(error){check(error instanceof TypeError&&gets===0,'private-brand-fails-before-rhs-adoption');}
  var writes=[],seenThis;
  class Parent{get chosen(){seenThis=this;return 0;}set chosen(value){writes.push(['parent',this,value]);}}
  class Other{set chosen(value){writes.push(['other',this,value]);}}
  class Child extends Parent{async choose(key,rhs){return super[await key] ||= await rhs;}}
  var child=new Child();waiting=new Promise(function(resolve){release=resolve;});
  completion=child.choose('chosen',waiting);await 0;await 0;
  check(seenThis===child,'super-get-uses-original-this');Object.setPrototypeOf(Child.prototype,Other.prototype);gc();release(whole);
  check(await completion===whole&&writes.length===1&&writes[0][0]==='parent'&&writes[0][1]===child&&writes[0][2]===whole,'super-put-does-not-reresolve-base-after-rhs');
  var eager={value:undefined};check(await (async function(){return (await eager).value ??= 17;})()===17&&eager.value===17,'awaited-left-eager-rhs-has-only-left-suspension');
}
run().then(function(){print('logical-assignment-complete-references:ok');},function(error){print(error);throw error;});
