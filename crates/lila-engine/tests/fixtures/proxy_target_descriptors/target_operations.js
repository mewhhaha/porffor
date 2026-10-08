const marker = new Error('target-descriptor-marker');
const receiver = {};
function invoke(kind, proxy, incoming) {
  if (kind === 0) return Reflect.get(proxy, 'x', receiver);
  if (kind === 1) return Reflect.set(proxy, 'x', incoming, receiver);
  if (kind === 2) return Reflect.has(proxy, 'x');
  return Reflect.deleteProperty(proxy, 'x');
}
function handler(kind, result, trace) {
  if (kind === 0) return {get(target, key, actualReceiver) {
    if (key !== 'x' || actualReceiver !== receiver) throw 'get operands';
    trace.push('trap'); return result;
  }};
  if (kind === 1) return {set(target, key, value, actualReceiver) {
    if (key !== 'x' || actualReceiver !== receiver) throw 'set operands';
    trace.push('trap'); return result;
  }};
  if (kind === 2) return {has(target, key) {
    if (key !== 'x') throw 'has key';
    trace.push('trap'); return result;
  }};
  return {deleteProperty(target, key) {
    if (key !== 'x') throw 'delete key';
    trace.push('trap'); return result;
  }};
}
function expectTypeError(action) {
  try {action();} catch (error) {
    if (Object.getPrototypeOf(error) !== TypeError.prototype) throw 'target descriptor native TypeError';
    return;
  }
  throw 'missing target descriptor TypeError';
}
function expectMarker(action) {
  let result = 'before';
  let finallyRuns = 0;
  try {
    try {result = action();} finally {finallyRuns++;}
    throw 'missing target descriptor marker';
  } catch (error) {
    if (error !== marker) throw 'target descriptor original throw identity';
  }
  if (result !== 'before' || finallyRuns !== 1) throw 'target descriptor abrupt effects';
}
let cases = 0;
for (let kind = 0; kind < 4; kind++) {
  const trace = [];
  const base = {x: 1};
  const inner = new Proxy(base, {
    getOwnPropertyDescriptor(target, key) {
      if (target !== base || key !== 'x') throw 'descriptor target and key';
      trace.push('descriptor');
      return Reflect.getOwnPropertyDescriptor(target, key);
    },
    isExtensible(target) {
      trace.push('extensible');
      return Reflect.isExtensible(target);
    }
  });
  const result = kind === 0 ? 9 : kind === 2 ? false : true;
  const outer = new Proxy(inner, handler(kind, result, trace));
  if (invoke(kind, outer, 9) !== result || base.x !== 1) throw 'normal outer trap result or target mutation';
  // The inner descriptor protocol also checks its target extensibility.
  const expected = kind < 2 ? 'trap,descriptor' : 'trap,descriptor,extensible';
  if (trace.join(',') !== expected) throw 'outer invariant target operation order';

  const fixed = {};
  Object.defineProperty(fixed, 'x', {value: 1, writable: false, configurable: false});
  const throwingTrace = [];
  const throwingTarget = new Proxy(fixed, {getOwnPropertyDescriptor() {
    throwingTrace.push('descriptor'); throw marker;
  }});
  expectMarker(() => invoke(kind, new Proxy(throwingTarget, handler(kind, result, throwingTrace)), 9));
  if (throwingTrace.join(',') !== 'trap,descriptor') throw 'target throw precedes outer invariant';
  const honestFixed = new Proxy(fixed, {getOwnPropertyDescriptor(target, key) {
    return Reflect.getOwnPropertyDescriptor(target, key);
  }});
  expectTypeError(() => invoke(kind, new Proxy(honestFixed, handler(kind, result, [])), 9));
  const invalid = new Proxy({}, {getOwnPropertyDescriptor() {return 7;}});
  expectTypeError(() => invoke(kind, new Proxy(invalid, handler(kind, result, [])), 9));
  const hiddenFixed = new Proxy(fixed, {getOwnPropertyDescriptor() {return undefined;}});
  expectTypeError(() => invoke(kind, new Proxy(hiddenFixed, handler(kind, result, [])), 9));

  const absentTrace = [];
  const absent = new Proxy({}, {
    getOwnPropertyDescriptor() {absentTrace.push('descriptor'); return undefined;},
    isExtensible() {throw 'absent descriptor must skip outer extensibility';}
  });
  if (invoke(kind, new Proxy(absent, handler(kind, result, absentTrace)), 9) !== result) throw 'absent descriptor normal result';
  if (absentTrace.join(',') !== 'trap,descriptor') throw 'absent descriptor order';
  cases++;
}
// Boolean outcomes that cannot violate an invariant return before target reads.
for (let kind = 1; kind < 4; kind++) {
  const shortResult = kind === 2;
  const target = new Proxy({}, {getOwnPropertyDescriptor() {throw 'short result descriptor lookup';}});
  if (invoke(kind, new Proxy(target, handler(kind, shortResult, [])), 4) !== shortResult) throw 'short result Boolean';
}
const symbol = Symbol('target-descriptor-key');
for (let kind = 0; kind < 4; kind++) {
  const base = {};
  base[symbol] = 1;
  let descriptorCalls = 0;
  const inner = new Proxy(base, {getOwnPropertyDescriptor(target, key) {
    if (key !== symbol) throw 'target descriptor Symbol identity';
    descriptorCalls++;
    return Reflect.getOwnPropertyDescriptor(target, key);
  }});
  const traps = kind === 0 ? {get() {return 9;}}
    : kind === 1 ? {set() {return true;}}
    : kind === 2 ? {has() {return false;}}
    : {deleteProperty() {return true;}};
  const outer = new Proxy(inner, traps);
  const result = kind === 0 ? Reflect.get(outer, symbol, receiver)
    : kind === 1 ? Reflect.set(outer, symbol, 9, receiver)
    : kind === 2 ? Reflect.has(outer, symbol)
    : Reflect.deleteProperty(outer, symbol);
  const expected = kind === 0 ? 9 : kind === 2 ? false : true;
  if (result !== expected || descriptorCalls !== 1 || base[symbol] !== 1) throw 'Symbol target descriptor result';
  const throwingTraps = kind === 0 ? {get() {throw marker;}}
    : kind === 1 ? {set() {throw marker;}}
    : kind === 2 ? {has() {throw marker;}}
    : {deleteProperty() {throw marker;}};
  const unreadTarget = new Proxy({}, {getOwnPropertyDescriptor() {throw 'abrupt outer trap must skip descriptor';}});
  expectMarker(() => invoke(kind, new Proxy(unreadTarget, throwingTraps), 9));
}
const nanTarget = {};
Object.defineProperty(nanTarget, 'x', {value: NaN, writable: false, configurable: false});
const wrappedNaN = new Proxy(nanTarget, {});
if (!Number.isNaN(invoke(0, new Proxy(wrappedNaN, handler(0, NaN, [])), 0))) throw 'get SameValue NaN';
if (!invoke(1, new Proxy(wrappedNaN, handler(1, true, [])), NaN)) throw 'set SameValue NaN';
const zeroTarget = {};
Object.defineProperty(zeroTarget, 'x', {value: -0, writable: false, configurable: false});
const wrappedZero = new Proxy(zeroTarget, {});
expectTypeError(() => invoke(0, new Proxy(wrappedZero, handler(0, 0, [])), 0));
expectTypeError(() => invoke(1, new Proxy(wrappedZero, handler(1, true, [])), 0));
if (1 / invoke(0, new Proxy(wrappedZero, handler(0, -0, [])), 0) !== -Infinity) throw 'get SameValue negative zero';
if (!invoke(1, new Proxy(wrappedZero, handler(1, true, [])), -0)) throw 'set SameValue negative zero';
const accessorTarget = {};
Object.defineProperty(accessorTarget, 'x', {get: undefined, set: undefined, configurable: false});
const wrappedAccessor = new Proxy(accessorTarget, {});
if (invoke(0, new Proxy(wrappedAccessor, handler(0, undefined, [])), 0) !== undefined) throw 'missing getter allows undefined';
expectTypeError(() => invoke(0, new Proxy(wrappedAccessor, handler(0, 1, [])), 0));
expectTypeError(() => invoke(1, new Proxy(wrappedAccessor, handler(1, true, [])), 1));
if (cases !== 4) throw 'four target descriptor operations census';
print('proxy-target-descriptors:ok');
262;
