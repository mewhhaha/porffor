const trace = [];
const token = Symbol('generator receiver');
const marker = {kind: 'injected'};
marker.self = marker;
const getterMarker = Symbol('generator getter');
let selected;
let callable;
let gets = 0;
let mode = 'call';
let argumentReads = 0;
function first() {
  trace.push('first');
  selected = null;
  callable = function() { throw 'replacement'; };
  return 1;
}
function last() { trace.push('last'); argumentReads++; return 3; }
class C {
  constructor(value) { this.token = value; this.self = this; }
  get #callee() {
    gets++;
    trace.push('get');
    if (mode === 'throw') throw getterMarker;
    return callable;
  }
  *values(value) {
    try { return value.#callee?.((value = null, first()), yield 'argument', last()); }
    finally { trace.push('finally'); }
  }
  *missing(value) { return value.#callee?.(last(), yield 'forbidden'); }
}
const runner = new C(Symbol('runner'));
callable = new Proxy(function(a, b, c) {
  'use strict';
  trace.push('call');
  if (this.token !== token || this.self !== this || a !== 1 || b !== 2 || c !== 3) {
    throw 'private generator saved Reference';
  }
  return 31;
}, {apply(target, receiver, args) {
  trace.push('apply');
  if (receiver.token !== token || receiver.self !== receiver) throw 'generator Proxy receiver';
  return Reflect.apply(target, receiver, args);
}});
selected = new C(token);
let iterator = runner.values(selected);
let step = iterator.next();
if (step.done || step.value !== 'argument' || trace.join(',') !== 'get,first' || selected !== null) {
  throw 'private getter must run once before first yield';
}
gc();
step = iterator.next(2);
if (!step.done || step.value !== 31 || gets !== 1 || argumentReads !== 1 ||
    trace.join(',') !== 'get,first,last,apply,call,finally') {
  throw 'private callee and receiver roots across yield';
}

trace.length = 0;
callable = null;
step = runner.missing(runner).next();
if (!step.done || step.value !== undefined || argumentReads !== 1 || trace.join(',') !== 'get') {
  throw 'private optional callee must skip every argument and yield';
}
let observed;
const beforeGets = gets;
try { runner.missing(new Proxy(runner, {})).next(); } catch (error) { observed = error; }
if (!(observed instanceof TypeError) || gets !== beforeGets || argumentReads !== 1) {
  throw 'private brand failure before optional skip';
}
mode = 'throw';
trace.length = 0;
try { runner.missing(runner).next(); } catch (error) { observed = error; }
if (observed !== getterMarker || argumentReads !== 1 || trace.join(',') !== 'get') {
  throw 'private getter Throw must survive generator entry';
}
mode = 'call';
callable = function() { throw 'injected Throw reached callee'; };
trace.length = 0;
iterator = runner.values(runner);
step = iterator.next();
if (step.done || step.value !== 'argument') throw 'injected case must suspend';
gc();
try { iterator.throw(marker); } catch (error) { observed = error; }
if (observed !== marker || observed.self !== observed || argumentReads !== 1 ||
    trace.join(',') !== 'get,first,finally' || !iterator.next().done) {
  throw 'whole injected Throw, argument cutoff and one finally';
}
print('private-optional-yield:ok');
262;
