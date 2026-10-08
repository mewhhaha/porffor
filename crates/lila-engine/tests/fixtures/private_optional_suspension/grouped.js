const trace = [];
const marker = Symbol('outer argument');
let factory;
let owner;
const child = {method(value) {
  'use strict';
  trace.push('child');
  if (this !== child || value !== 7) throw 'grouped terminal property Reference';
  return 47;
}};
function returned(value) {
  'use strict';
  trace.push('returned');
  if (this !== undefined || value !== 7) throw 'grouped terminal Call must publish Value';
  return 37;
}
function inner() { trace.push('inner'); return 3; }
function outer() { trace.push('outer'); gc(); return 7; }
function throwingOuter() { trace.push('throwing-outer'); throw marker; }
class C {
  get #factory() { trace.push('get'); return factory; }
  static async call(value) { return (value.#factory?.(await inner()))(await outer()); }
  static async property(value) { return (value.#factory?.(await inner()).method)(await outer()); }
  static async abrupt(value) { return (value.#factory?.(await inner()))(await throwingOuter()); }
  *call(value) { return (value.#factory?.(yield 'inner'))(yield 'outer'); }
  *property(value) { return (value.#factory?.(yield 'inner').method)(yield 'outer'); }
}
function select(result) {
  factory = function(value) {
    'use strict';
    trace.push('factory');
    if (this !== owner || value !== 3) throw 'inner private factory receiver';
    factory = function() { throw 'factory reread'; };
    return result;
  };
}
async function run() {
  owner = new C();
  select(returned);
  if (await C.call(owner) !== 37 || trace.join(',') !== 'get,inner,factory,outer,returned') {
    throw 'awaited grouped private Call Value';
  }
  trace.length = 0;
  select(child);
  if (await C.property(owner) !== 47 || trace.join(',') !== 'get,inner,factory,outer,child') {
    throw 'awaited grouped private public-property Reference';
  }
  trace.length = 0;
  factory = undefined;
  let observed;
  try { await C.call(owner); } catch (error) { observed = error; }
  if (!(observed instanceof TypeError) || trace.join(',') !== 'get,outer') {
    throw 'grouped nullish Value must still evaluate outer argument before Call';
  }
  trace.length = 0;
  try { await C.abrupt(owner); } catch (error) { observed = error; }
  if (observed !== marker || trace.join(',') !== 'get,throwing-outer') {
    throw 'grouped outer argument Throw precedes callability';
  }

  trace.length = 0;
  select(returned);
  let iterator = owner.call(owner);
  let step = iterator.next();
  if (step.done || step.value !== 'inner' || trace.join(',') !== 'get') throw 'inner yield';
  step = iterator.next(3);
  if (step.done || step.value !== 'outer' || trace.join(',') !== 'get,factory') throw 'outer yield';
  gc();
  factory = null;
  step = iterator.next(7);
  if (!step.done || step.value !== 37 || trace.join(',') !== 'get,factory,returned') {
    throw 'yielded grouped private Call Value';
  }
  trace.length = 0;
  select(child);
  iterator = owner.property(owner);
  if (iterator.next().value !== 'inner' || iterator.next(3).value !== 'outer') throw 'property yields';
  gc();
  step = iterator.next(7);
  if (!step.done || step.value !== 47 || trace.join(',') !== 'get,factory,child') {
    throw 'yielded grouped private public-property Reference';
  }
  trace.length = 0;
  factory = null;
  iterator = owner.call(owner);
  step = iterator.next();
  if (step.done || step.value !== 'outer' || trace.join(',') !== 'get') {
    throw 'nullish inner skips inner yield but reaches ordinary outer argument';
  }
  try { iterator.next(7); } catch (error) { observed = error; }
  if (!(observed instanceof TypeError) || trace.join(',') !== 'get') throw 'outer Call TypeError';
}
run().then(() => print('private-optional-grouped:ok'), error => print('unexpected:' + error));
262;
