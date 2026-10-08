const trace = [];
const fixedSymbol = Symbol('fixed');
const base = {loose: 1};
Object.defineProperty(base, 'fixed', {value: 2, configurable: false});
Object.defineProperty(base, fixedSymbol, {value: 3, configurable: false});
const target = new Proxy(base, {
  isExtensible(actual) {
    trace.push('extensible');
    return Reflect.isExtensible(actual);
  },
  ownKeys(actual) {
    trace.push('targetKeys');
    return ['fixed', 'loose', fixedSymbol];
  },
  getOwnPropertyDescriptor(actual, key) {
    trace.push(key === fixedSymbol ? 'descriptor:symbol' : 'descriptor:' + key);
    return Reflect.getOwnPropertyDescriptor(actual, key);
  }
});
const list = {get length() {trace.push('length'); return 3;},
  get 0() {trace.push('0'); return fixedSymbol;},
  get 1() {trace.push('1'); return 'extra';},
  get 2() {trace.push('2'); return ['fi', 'xed'].join('');}};
const outer = new Proxy(target, {ownKeys(actual) {
  if (actual !== target) throw 'outer target identity';
  trace.push('outer');
  return list;
}});
const result = Reflect.ownKeys(outer);
if (result.length !== 3 || result[0] !== fixedSymbol || result[1] !== 'extra' || result[2] !== 'fixed') throw 'outer snapshot publication';
if (trace.join(',') !== 'outer,length,0,1,2,extensible,targetKeys,descriptor:fixed,descriptor:loose,descriptor:symbol') throw 'ordered complete target operations';

const other = __lilaCreateRealm().global;
const marker = new other.Error('target-descriptor-marker');
function expectMarker(action) {
  try {action();} catch (error) {
    if (error !== marker) throw 'target abrupt identity';
    return;
  }
  throw 'missing target marker';
}
const abruptTrace = [];
const abruptBase = {later: 1};
Object.defineProperty(abruptBase, 'fixed', {value: 2, configurable: false});
const abruptTarget = new Proxy(abruptBase, {
  isExtensible(actual) {abruptTrace.push('extensible'); return true;},
  ownKeys(actual) {abruptTrace.push('targetKeys'); return ['fixed', 'later'];},
  getOwnPropertyDescriptor(actual, key) {
    abruptTrace.push('descriptor:' + key);
    if (key === 'later') throw marker;
    return Reflect.getOwnPropertyDescriptor(actual, key);
  }
});
let finallyRuns = 0;
let assignment = 'before';
expectMarker(() => {
  try {
    assignment = Reflect.ownKeys(new Proxy(abruptTarget, {ownKeys() {return [];}}));
  } finally {finallyRuns++;}
});
if (assignment !== 'before' || finallyRuns !== 1 || abruptTrace.join(',') !== 'extensible,targetKeys,descriptor:fixed,descriptor:later') throw 'later descriptor abrupt precedes missing fixed key';

const earlyTrace = [];
const earlyTarget = new Proxy({}, {
  isExtensible() {earlyTrace.push('extensible'); throw marker;},
  ownKeys() {earlyTrace.push('targetKeys'); throw 'after extensibility abrupt';},
  getOwnPropertyDescriptor() {throw 'after extensibility abrupt descriptor';}
});
expectMarker(() => Reflect.ownKeys(new Proxy(earlyTarget, {ownKeys() {return [];}})));
if (earlyTrace.join(',') !== 'extensible') throw 'extensibility abrupt order';
const keyTrace = [];
const keyTarget = new Proxy({}, {
  isExtensible() {keyTrace.push('extensible'); return true;},
  ownKeys() {keyTrace.push('targetKeys'); throw marker;},
  getOwnPropertyDescriptor() {throw 'after target keys abrupt';}
});
expectMarker(() => Object.getOwnPropertyNames(new Proxy(keyTarget, {ownKeys() {return [];}})));
if (keyTrace.join(',') !== 'extensible,targetKeys') throw 'target keys abrupt order';

let untouchedTargetCalls = 0;
const untouchedTarget = new Proxy({}, {
  isExtensible() {untouchedTargetCalls++; throw 'duplicate touched target';},
  ownKeys() {untouchedTargetCalls++; throw 'duplicate touched target keys';}
});
try {
  Reflect.ownKeys(new Proxy(untouchedTarget, {ownKeys() {return ['same', 'same'];}}));
  throw 'missing duplicate TypeError';
} catch (error) {
  if (Object.getPrototypeOf(error) !== TypeError.prototype) throw error;
}
if (untouchedTargetCalls !== 0) throw 'duplicates precede target operations';

const mutationTrace = [];
const mutationBase = {key: 1};
const mutationTarget = new Proxy(mutationBase, {
  isExtensible(actual) {mutationTrace.push('extensible'); return Reflect.isExtensible(actual);},
  ownKeys(actual) {
    mutationTrace.push('targetKeys');
    Object.preventExtensions(actual);
    return ['key'];
  },
  getOwnPropertyDescriptor(actual, key) {
    mutationTrace.push('descriptor:' + key);
    return Reflect.getOwnPropertyDescriptor(actual, key);
  }
});
const mutationResult = Reflect.ownKeys(new Proxy(mutationTarget, {ownKeys() {return ['extra', 'key'];}}));
if (mutationResult.length !== 2 || mutationResult[0] !== 'extra' || mutationResult[1] !== 'key' || Object.isExtensible(mutationBase)) throw 'captured extensibility result';
if (mutationTrace.join(',') !== 'extensible,targetKeys,descriptor:key') throw 'extensibility is not rechecked after target keys';

const snapshotTrace = [];
const snapshotBase = {first: 1, removed: 2};
Object.preventExtensions(snapshotBase);
const snapshotTarget = new Proxy(snapshotBase, {
  isExtensible(actual) {snapshotTrace.push('extensible'); return Reflect.isExtensible(actual);},
  ownKeys(actual) {snapshotTrace.push('targetKeys'); return ['first', 'removed'];},
  getOwnPropertyDescriptor(actual, key) {
    snapshotTrace.push('descriptor:' + key);
    if (key === 'first') delete actual.removed;
    return Reflect.getOwnPropertyDescriptor(actual, key);
  }
});
const snapshotResult = Reflect.ownKeys(new Proxy(snapshotTarget, {ownKeys() {return ['removed', 'first'];}}));
if (snapshotResult.length !== 2 || snapshotResult[0] !== 'removed' || snapshotResult[1] !== 'first' || Object.hasOwn(snapshotBase, 'removed')) throw 'captured target keys survive descriptor mutation';
if (snapshotTrace.join(',') !== 'extensible,targetKeys,descriptor:first,descriptor:removed') throw 'all captured target descriptors are visited';
print('proxy-own-keys-target-operations:ok');
262;
