const trace = [];
const token = Symbol('private receiver');
const getterMarker = Symbol('private GetValue Throw');
const argumentMarker = {kind: 'argument'};
argumentMarker.self = argumentMarker;
let flag = false;
let holder = {value: 1};
let selected;
let callable;
let mode = 'value';
let reads = 0;
let argumentsRead = 0;
function argument() {
  trace.push('argument');
  argumentsRead++;
  if (!flag || holder.value !== 2) throw 'eager argument used pre-getter caller facts';
  selected = null;
  callable = function() { throw 'callee reread'; };
  gc();
  return 2;
}
function throwingArgument() {
  argumentsRead++;
  trace.push('argument-throw');
  throw argumentMarker;
}
class C {
  constructor(value) { this.token = value; this.self = this; }
  get #value() {
    reads++;
    trace.push('value-get');
    flag = true;
    holder = {value: 2};
    if (mode === 'throw') throw getterMarker;
    return 17;
  }
  get #callee() {
    reads++;
    trace.push('callee-get');
    flag = true;
    holder = {value: 2};
    if (mode === 'throw') throw getterMarker;
    return callable;
  }
  #known(value) { return value + 1; }
  known(value) { return this.#known(value); }
  static read(value) {
    const result = value.#value;
    if (!flag || holder.value !== 2) throw 'syntax read left stale caller facts';
    return result;
  }
  static call(value) { return value.#callee(argument()); }
  static throwing(value) { return value.#callee(throwingArgument()); }
}
if (new C(Symbol('known receiver')).known(41) !== 42) throw 'proven private method result';
selected = new C(token);
if (C.read(selected) !== 17 || reads !== 1 || trace.join(',') !== 'value-get') {
  throw 'ordinary private GetValue was repeated or lost its result';
}
trace.length = 0;
flag = false;
holder = {value: 1};
callable = new Proxy(function(value) {
  'use strict';
  trace.push('call');
  if (this.token !== token || this.self !== this || value !== 2) throw 'ordinary private saved receiver';
  return 42;
}, {apply(target, receiver, args) {
  trace.push('apply');
  if (receiver.token !== token || receiver.self !== receiver) throw 'ordinary private Proxy receiver';
  return Reflect.apply(target, receiver, args);
}});
if (C.call(selected) !== 42 || reads !== 2 || argumentsRead !== 1 || selected !== null ||
    trace.join(',') !== 'callee-get,argument,apply,call') {
  throw 'ordinary private callee effects/acquisition before arguments';
}

const owner = new C(Symbol('other receiver'));
mode = 'throw';
trace.length = 0;
let observed;
try { C.call(owner); } catch (error) { observed = error; }
if (observed !== getterMarker || argumentsRead !== 1 || trace.join(',') !== 'callee-get') {
  throw 'private GetValue Throw must precede every ordinary argument';
}
mode = 'value';
callable = 17;
trace.length = 0;
try { C.throwing(owner); } catch (error) { observed = error; }
if (observed !== argumentMarker || observed.self !== observed || argumentsRead !== 2 ||
    trace.join(',') !== 'callee-get,argument-throw') {
  throw 'whole argument Throw must precede private callee callability';
}
const before = reads;
try { C.call(new Proxy(owner, {})); } catch (error) { observed = error; }
if (!(observed instanceof TypeError) || reads !== before || argumentsRead !== 2) {
  throw 'private brand must precede ordinary getter effects and arguments';
}
print('private-get-value-effects:ok');
262;
