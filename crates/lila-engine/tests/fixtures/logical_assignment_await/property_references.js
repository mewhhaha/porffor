const trace = [];
let skippedCalls = 0;
function skipped() { skippedCalls++; throw 'skipped property RHS'; }
async function run() {
  const key = Symbol('key');
  const laterKey = Symbol('later');
  let slot = 0;
  let proxy;
  const target = {};
  Object.defineProperty(target, key, {
    get() { if (this !== proxy) throw 'getter receiver'; trace.push('getter'); return slot; },
    set(value) { if (this !== proxy) throw 'setter receiver'; trace.push('setter'); slot = value; }
  });
  proxy = new Proxy(target, {
    get(object, name, receiver) {
      if (object !== target || name !== key || receiver !== proxy) throw 'get Reference';
      trace.push('get'); return Reflect.get(object, name, receiver);
    },
    set(object, name, value, receiver) {
      if (object !== target || name !== key || receiver !== proxy || value !== 17) throw 'set Reference';
      trace.push('set'); return Reflect.set(object, name, value, receiver);
    }
  });
  const replacement = {[key]: 99, [laterKey]: 88};
  let current = proxy;
  let keyCalls = 0;
  const keyObject = {[Symbol.toPrimitive](hint) {
    if (hint !== 'string') throw 'key hint';
    trace.push('coerce'); keyCalls++; return key;
  }};
  function base() { trace.push('base'); return current; }
  function keySource() { trace.push('key-source'); return keyObject; }
  const thenable = {get then() {
    trace.push('then'); current = replacement;
    return resolve => { trace.push('resolve'); resolve(17); };
  }};
  function rhs() {
    trace.push('rhs'); slot = 55;
    keyObject[Symbol.toPrimitive] = function() { throw 'key converted again'; };
    return thenable;
  }
  const result = base()[keySource()] ||= await rhs();
  if (result !== 17 || slot !== 17 || replacement[key] !== 99 || replacement[laterKey] !== 88 || keyCalls !== 1) throw 'retained property Reference';
  if (trace.join(',') !== 'base,key-source,coerce,get,getter,rhs,then,caller,resolve,set,setter') throw 'capture Get await Put order';
  trace.length = 0;
  if ((proxy[key] ||= await skipped()) !== 17 || trace.join(',') !== 'get,getter' || skippedCalls !== 0) throw 'property skip has no Put';
  let primitiveGets = 0;
  let primitiveSet;
  Object.defineProperty(Number.prototype, 'logicalAwaitValue', {
    configurable: true,
    get: function() { 'use strict'; if (this !== 0) throw 'primitive get receiver'; primitiveGets++; return undefined; },
    set: function(value) { 'use strict'; if (this !== 0) throw 'primitive set receiver'; primitiveSet = value; }
  });
  const primitiveResult = (0).logicalAwaitValue ??= await 23;
  if (primitiveResult !== 23 || primitiveSet !== 23 || primitiveGets !== 1) throw 'primitive original receiver';
  delete Number.prototype.logicalAwaitValue;
  const holder = {value: 0};
  const table = {holder};
  const outer = (table?.holder).value ||= await 31;
  if (outer !== 31 || holder.value !== 31) throw 'ordinary Reference over optional value';
  const stringResult = 'a'[0] ||= await skipped();
  if (stringResult !== 'a' || skippedCalls !== 0) throw 'virtual string property skipped write';
}
run().then(() => print('logical-assignment-properties:ok'), error => print('unexpected:' + error));
trace.push('caller');
