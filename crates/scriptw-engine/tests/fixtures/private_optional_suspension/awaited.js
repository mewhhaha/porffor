const trace = [];
const token = Symbol('receiver');
const getterMarker = Symbol('private getter');
const argumentMarker = {kind: 'argument'};
argumentMarker.self = argumentMarker;
let selected;
let callable;
let mode = 'call';
let gets = 0;
let argumentsRead = 0;
let flag = false;
let captured = {value: 1};
function first() {
  trace.push('first');
  selected = null;
  callable = function() { throw 'replacement'; };
  return 1;
}
function last() { trace.push('last'); return 3; }
function argument() { argumentsRead++; trace.push('argument'); return 1; }
function throwingArgument() { argumentsRead++; throw argumentMarker; }
const pending = {get then() {
  trace.push('then');
  gc();
  return resolve => { trace.push('resolve'); gc(); resolve(2); };
}};
class C {
  constructor(value) { this.token = value; this.self = this; }
  get #callee() {
    gets++;
    trace.push('get');
    flag = true;
    captured.value = 2;
    if (mode === 'throw') throw getterMarker;
    return callable;
  }
  static async invoke(value) {
    return value.#callee?.((value = null, first()), await pending, last());
  }
  static async check(value) { return value.#callee?.(argument(), await 0); }
  static async rejected(value) {
    try { return value.#callee?.(await Promise.reject(argumentMarker), last()); }
    finally { trace.push('finally'); }
  }
  static async noncallable(value) { return value.#callee?.(throwingArgument(), await 0); }
}
async function wrongBrand(value) {
  let observed;
  const before = argumentsRead;
  const beforeGets = gets;
  try { await C.check(value); } catch (error) { observed = error; }
  if (!(observed instanceof TypeError) || argumentsRead !== before || gets !== beforeGets) {
    throw 'private brand must precede optional callee and every argument';
  }
}
async function run() {
  callable = new Proxy(function(a, b, c) {
    'use strict';
    trace.push('call');
    if (this.token !== token || this.self !== this || a !== 1 || b !== 2 || c !== 3) {
      throw 'captured private receiver or arguments';
    }
    return 31;
  }, {apply(target, receiver, args) {
    trace.push('apply');
    if (receiver.token !== token || receiver.self !== receiver) throw 'Proxy raw receiver';
    return Reflect.apply(target, receiver, args);
  }});
  selected = new C(token);
  const result = await C.invoke(selected);
  if (result !== 31 || gets !== 1 || selected !== null || !flag || captured.value !== 2 ||
      trace.join(',') !== 'get,first,then,resolve,last,apply,call') {
    throw 'private getter effects, acquired callee or await root ordering';
  }

  trace.length = 0;
  flag = false;
  captured = {value: 1};
  callable = undefined;
  const owner = new C(Symbol('other receiver'));
  if (await C.check(owner) !== undefined || argumentsRead !== 0 || !flag ||
      captured.value !== 2 || trace.join(',') !== 'get') {
    throw 'nullish callee must retain getter effects and skip complete arguments';
  }
  await wrongBrand({});
  await wrongBrand(null);
  await wrongBrand(new Proxy(owner, {}));

  mode = 'throw';
  trace.length = 0;
  let observed;
  try { await C.check(owner); } catch (error) { observed = error; }
  if (observed !== getterMarker || argumentsRead !== 0 || trace.join(',') !== 'get') {
    throw 'private getter Throw identity or argument cutoff';
  }
  mode = 'call';
  callable = function() { throw 'rejected argument called callee'; };
  trace.length = 0;
  try { await C.rejected(owner); } catch (error) { observed = error; }
  if (observed !== argumentMarker || observed.self !== observed ||
      trace.join(',') !== 'get,finally') {
    throw 'whole rejected argument Throw and finally';
  }
  callable = 17;
  try { await C.noncallable(owner); } catch (error) { observed = error; }
  if (observed !== argumentMarker || argumentsRead !== 1) {
    throw 'argument Throw must precede non-nullish callability';
  }
}
run().then(() => print('private-optional-await:ok'), error => print('unexpected:' + error));
262;
