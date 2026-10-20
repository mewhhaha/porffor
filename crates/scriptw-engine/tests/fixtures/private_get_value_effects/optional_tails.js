const trace = [];
const token = Symbol('returned receiver');
const marker = Symbol('private Value Throw');
let selected;
let callable;
let flag = false;
let holder = {value: 1};
let mode = 'value';
let reads = 0;
let forbiddenReads = 0;
function first() {
  trace.push('first');
  if (!flag || holder.value !== 2) throw 'private Value getter effects before arguments';
  selected = null;
  callable = function() { throw 'returned method reread'; };
  return 1;
}
function last() { trace.push('last'); return 3; }
function forbidden() { forbiddenReads++; throw marker; }
function key() { trace.push('key'); selected = null; gc(); return 'method'; }
function outer() {
  trace.push('outer');
  callable = function() { throw 'grouped method reread'; };
  gc();
  return 7;
}
const pending = {get then() {
  trace.push('then'); gc();
  return resolve => { trace.push('resolve'); gc(); resolve(2); };
}};
class C {
  get #value() {
    reads++;
    trace.push('private');
    flag = true;
    holder = {value: 2};
    if (mode === 'throw') throw marker;
    if (mode === 'missing') return null;
    return selected;
  }
  static async call(value) {
    return value.#value?.method((value = null, first()), await pending, last());
  }
  static async missing(value) { return value.#value?.method(await forbidden()); }
  static async grouped(value) { return (value.#value?.[await key()])(await outer()); }
  *call(value) { return value.#value?.method((value = null, first()), yield 'argument', last()); }
  *missing(value) { return value.#value?.method(yield forbidden()); }
  *grouped(value) { return (value.#value?.[yield 'key'])(yield 'outer'); }
}
function select(grouped) {
  selected = {token, get method() { trace.push('method'); return callable; }};
  selected.self = selected;
  callable = new Proxy(function(a, b, c) {
    'use strict';
    trace.push('call');
    if (this.token !== token || this.self !== this) throw 'returned Value property receiver';
    if (grouped) {
      if (a !== 7) throw 'grouped argument';
      return 47;
    }
    if (a !== 1 || b !== 2 || c !== 3) throw 'returned Value arguments';
    return 31;
  }, {apply(target, receiver, args) {
    trace.push('apply');
    if (receiver.token !== token || receiver.self !== receiver) throw 'returned Value Proxy receiver';
    return Reflect.apply(target, receiver, args);
  }});
}
async function run() {
  select(false);
  if (await C.call(new C()) !== 31 || reads !== 1 || selected !== null ||
      trace.join(',') !== 'private,method,first,then,resolve,last,apply,call') {
    throw 'awaited private Value acquisition and returned receiver roots';
  }
  trace.length = 0;
  flag = false;
  holder = {value: 1};
  select(false);
  const runner = new C();
  let iterator = runner.call(new C());
  let step = iterator.next();
  if (step.done || step.value !== 'argument' || !flag || holder.value !== 2 ||
      trace.join(',') !== 'private,method,first') throw 'private Value before generator yield';
  gc();
  step = iterator.next(2);
  if (!step.done || step.value !== 31 || reads !== 2 ||
      trace.join(',') !== 'private,method,first,last,apply,call') {
    throw 'yielded private Value returned receiver roots';
  }

  mode = 'missing';
  trace.length = 0;
  flag = false;
  holder = {value: 1};
  if (await C.missing(runner) !== undefined || !flag || holder.value !== 2 ||
      forbiddenReads !== 0 || trace.join(',') !== 'private') {
    throw 'nullish private Value keeps getter effects and skips awaited tail';
  }
  trace.length = 0;
  step = runner.missing(runner).next();
  if (!step.done || step.value !== undefined || forbiddenReads !== 0 || trace.join(',') !== 'private') {
    throw 'nullish private Value skips generator tail and yield';
  }
  mode = 'throw';
  trace.length = 0;
  let observed;
  try { await C.missing(runner); } catch (error) { observed = error; }
  if (observed !== marker || forbiddenReads !== 0 || trace.join(',') !== 'private') {
    throw 'private Value getter Throw before optional choice';
  }
  trace.length = 0;
  try { runner.missing(runner).next(); } catch (error) { observed = error; }
  if (observed !== marker || forbiddenReads !== 0 || trace.join(',') !== 'private') {
    throw 'private Value whole getter Throw before generator choice';
  }

  mode = 'value';
  trace.length = 0;
  select(true);
  if (await C.grouped(runner) !== 47 || trace.join(',') !== 'private,key,method,outer,apply,call') {
    throw 'grouped awaited terminal Reference from private Value';
  }
  trace.length = 0;
  select(true);
  iterator = runner.grouped(runner);
  step = iterator.next();
  if (step.done || step.value !== 'key' || trace.join(',') !== 'private') throw 'grouped key yield';
  selected = null;
  gc();
  step = iterator.next('method');
  if (step.done || step.value !== 'outer' || trace.join(',') !== 'private,method') throw 'grouped outer yield';
  callable = function() { throw 'grouped saved callee reread'; };
  gc();
  step = iterator.next(7);
  if (!step.done || step.value !== 47 || trace.join(',') !== 'private,method,apply,call') {
    throw 'grouped yielded terminal Reference from private Value';
  }

  mode = 'missing';
  trace.length = 0;
  try { await C.grouped(runner); } catch (error) { observed = error; }
  if (!(observed instanceof TypeError) || trace.join(',') !== 'private,outer') {
    throw 'nullish private Value grouping still evaluates ordinary outer argument';
  }
  trace.length = 0;
  iterator = runner.grouped(runner);
  step = iterator.next();
  if (step.done || step.value !== 'outer' || trace.join(',') !== 'private') {
    throw 'nullish private Value grouping skips key but reaches outer yield';
  }
  try { iterator.next(7); } catch (error) { observed = error; }
  if (!(observed instanceof TypeError) || trace.join(',') !== 'private') throw 'grouped outer TypeError';
}
run().then(() => print('private-get-value-tails:ok'), error => print('unexpected:' + error));
262;
