const trace = [];
const symbol = Symbol('array-key');
const array = ['backing-zero', , 'backing-two'];
const inherited = Object.create(Array.prototype);
Object.defineProperty(inherited, '1', {get() {
  if (this !== array) throw 'inherited Array receiver';
  trace.push('inherited:1');
  return symbol;
}});
Object.setPrototypeOf(array, inherited);
Object.defineProperty(array, '0', {get() {
  if (this !== array) throw 'Array zero receiver';
  trace.push('array:0');
  return 'first';
}});
Object.defineProperty(array, '2', {get() {
  if (this !== array) throw 'Array two receiver';
  trace.push('array:2');
  return 'last';
}});
const target = {};
let trapCalls = 0;
const handler = {ownKeys(actual) {
  if (this !== handler || actual !== target || arguments.length !== 1) throw 'trap arguments';
  trapCalls++;
  return array;
}};
const proxy = new Proxy(target, handler);
const keys = Reflect.ownKeys(proxy);
const names = Object.getOwnPropertyNames(proxy);
const symbols = Object.getOwnPropertySymbols(proxy);
if (keys === array || keys.length !== 3 || keys[0] !== 'first' || keys[1] !== symbol || keys[2] !== 'last') throw 'Array live Gets';
if (names.length !== 2 || names[0] !== 'first' || names[1] !== 'last') throw 'name filtering';
if (symbols.length !== 1 || symbols[0] !== symbol) throw 'symbol filtering';
if (keys === names || keys === symbols || names === symbols) throw 'fresh publications';
if (Object.getPrototypeOf(keys) !== Array.prototype || Object.getPrototypeOf(names) !== Array.prototype || Object.getPrototypeOf(symbols) !== Array.prototype) throw 'local array prototypes';
if (trapCalls !== 3 || trace.join(',') !== 'array:0,inherited:1,array:2,array:0,inherited:1,array:2,array:0,inherited:1,array:2') throw 'once-only Array Gets';

const liveTrace = [];
let lengthReads = 0;
let lengthConversions = 0;
let middle = 'old-middle';
const parent = {get 2() {
  if (this !== list) throw 'generic inherited receiver';
  liveTrace.push('2');
  return 'inherited-last';
}};
const list = Object.create(parent);
Object.defineProperty(list, 'length', {configurable: true, get() {
  if (this !== list) throw 'length receiver';
  lengthReads++;
  liveTrace.push('length');
  return {valueOf() {
    lengthConversions++;
    liveTrace.push('ToLength');
    return 3.8;
  }};
}});
Object.defineProperty(list, '0', {configurable: true, get() {
  if (this !== list) throw 'generic zero receiver';
  liveTrace.push('0');
  middle = 'new-middle';
  Object.defineProperty(list, 'length', {value: 0});
  return 'captured-first';
}});
Object.defineProperty(list, '1', {get() {
  if (this !== list) throw 'generic one receiver';
  liveTrace.push('1');
  Object.defineProperty(list, '0', {value: 'mutated-first'});
  delete list[2];
  return middle;
}});
Object.defineProperty(list, '2', {value: 'deleted-own-last', writable: true, configurable: true});
const live = Reflect.ownKeys(new Proxy({}, {ownKeys() { return list; }}));
if (live.length !== 3 || live[0] !== 'captured-first' || live[1] !== 'new-middle' || live[2] !== 'inherited-last') throw 'live ascending Gets and snapshot';
if (list.length !== 0 || list[0] !== 'mutated-first') throw 'source mutation';
if (lengthReads !== 1 || lengthConversions !== 1 || liveTrace.join(',') !== 'length,ToLength,0,1,2') throw 'captured length and Get order';
const resultTrace = [];
const resultTarget = {length: 2, 0: 'proxy-first', 1: symbol};
let resultProxy;
resultProxy = new Proxy(resultTarget, {get(actual, key, receiver) {
  if (actual !== resultTarget || receiver !== resultProxy) throw 'Proxy result Get receiver';
  resultTrace.push(key);
  return Reflect.get(actual, key, receiver);
}});
const observedKeys = Reflect.ownKeys(new Proxy({}, {ownKeys() { return resultProxy; }}));
if (observedKeys.length !== 2 || observedKeys[0] !== 'proxy-first' || observedKeys[1] !== symbol) throw 'Proxy result snapshot';
if (resultTrace.join(',') !== 'length,0,1') throw 'once-only Proxy result Gets';
print('proxy-own-keys-live:ok');
262;
