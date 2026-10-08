const other = __lilaCreateRealm().global;
const LocalArray = Array;
const ForeignArray = other.Array;
const localPrototype = LocalArray.prototype;
const foreignPrototype = ForeignArray.prototype;
const localMethods = [localPrototype.toReversed, localPrototype.with, localPrototype.toSpliced];
const foreignMethods = [foreignPrototype.toReversed, foreignPrototype.with, foreignPrototype.toSpliced];
const hasOwn = Object.prototype.hasOwnProperty;
let forbiddenGets = 0;
function forbidden() { forbiddenGets++; throw 'constructor or species Get'; }
function wrongArray() { throw 'mutable Array constructor'; }
Object.defineProperty(LocalArray, Symbol.species, {configurable: true, get: forbidden});
Object.defineProperty(ForeignArray, Symbol.species, {configurable: true, get: forbidden});
Object.defineProperty(localPrototype, 'constructor', {configurable: true, get: forbidden});
Object.defineProperty(foreignPrototype, 'constructor', {configurable: true, get: forbidden});
globalThis.Array = wrongArray;
other.Array = wrongArray;

function check(result, prototype, expected) {
  if (Object.getPrototypeOf(result) !== prototype || !LocalArray.isArray(result)) throw 'defining-Realm Array';
  if (result.length !== expected.length) throw 'result length';
  for (let i = 0; i < expected.length; i++) {
    if (result[i] !== expected[i] || !hasOwn.call(result, i)) throw 'own copied element';
  }
}
function observe(target, trace) {
  const proxy = new Proxy(target, {
    get(object, key, receiver) {
      if (receiver !== proxy) throw 'original Proxy receiver';
      if (key === 'constructor' || key === Symbol.species) return forbidden();
      trace.push(key);
      return Reflect.get(object, key, receiver);
    },
    has() { throw 'by-copy HasProperty'; }
  });
  return proxy;
}

let borrowedCopies = 0;
for (let direction = 0; direction < 2; direction++) {
  const Source = direction === 0 ? ForeignArray : LocalArray;
  const methods = direction === 0 ? localMethods : foreignMethods;
  const prototype = direction === 0 ? localPrototype : foreignPrototype;
  const source = new Source(3);
  source[0] = 3; source[1] = 1; source[2] = 2;
  Object.defineProperty(source, 'constructor', {get: forbidden});
  check(methods[0].call(source), prototype, [2, 1, 3]);
  check(methods[1].call(source, 1, 9), prototype, [3, 9, 2]);
  check(methods[2].call(source, 1, 1, 9), prototype, [3, 9, 2]);
  if (source.length !== 3 || source[0] !== 3 || source[1] !== 1 || source[2] !== 2) throw 'source untouched';
  borrowedCopies += 3;
  check(methods[0].call(new Source()), prototype, []);
  check(methods[2].call(new Source(), 0, 0), prototype, []);
  check(methods[1].call({length: 1, get 0() { throw 'replacement Get'; }}, 0, 7), prototype, [7]);

  const inherited = Object.create(Source.prototype);
  Object.defineProperty(inherited, '1', {value: 7, configurable: true});
  const sparse = new Source(3);
  sparse[2] = 5;
  Object.setPrototypeOf(sparse, inherited);
  check(methods[0].call(sparse), prototype, [5, 7, undefined]);
  check(methods[1].call(sparse, 1, 9), prototype, [undefined, 9, 5]);
  check(methods[2].call(sparse, 1, 1), prototype, [undefined, 5]);
  if (hasOwn.call(sparse, 0) || hasOwn.call(sparse, 1) || sparse.length !== 3) throw 'holes remain in source';
  const generic = {length: 2, 0: 'left', 1: 'right', get constructor() { return forbidden(); }};
  check(methods[0].call(generic), prototype, ['right', 'left']);
  check(methods[1].call(generic, -1, 'new'), prototype, ['left', 'new']);
  check(methods[2].call(generic, 1, 0, 'middle'), prototype, ['left', 'middle', 'right']);
  check(methods[0].call('abc'), prototype, ['c', 'b', 'a']);

  const reverseTrace = [];
  let reverseLength = 3;
  let middle = 20;
  const reverseTarget = {
    get length() { return reverseLength; },
    get 0() { reverseTrace.push('get0'); return 10; },
    get 1() { reverseTrace.push('get1'); return middle; },
    get 2() { reverseTrace.push('get2'); middle = 90; reverseLength = 99; return 30; }
  };
  check(methods[0].call(observe(reverseTarget, reverseTrace)), prototype, [30, 90, 10]);
  if (reverseTrace.join(',') !== 'length,2,get2,1,get1,0,get0') throw 'descending live Get once';

  const withTrace = [];
  let withLength = 3;
  const replacement = {[Symbol.toPrimitive]() { throw 'replacement coercion'; }};
  const withTarget = {
    get length() { return withLength; },
    get 0() { withTrace.push('get0'); return 'zero'; },
    get 1() { throw 'replaced property Get'; },
    get 2() { withTrace.push('get2'); return 'two'; }
  };
  const index = {[Symbol.toPrimitive](hint) {
    if (hint !== 'number') throw 'index hint';
    withTrace.push('index'); withLength = 0; return -2;
  }};
  const withArgument = (label, value) => { withTrace.push(label); return value; };
  check(methods[1].call(observe(withTarget, withTrace), withArgument('argument-index', index), withArgument('argument-value', replacement)), prototype, ['zero', replacement, 'two']);
  if (withTrace.join(',') !== 'argument-index,argument-value,length,index,0,get0,2,get2') throw 'arguments then captured length and replacement skip';

  const spliceTrace = [];
  let spliceLength = 4;
  let finalValue = 'old';
  const insertion = {[Symbol.toPrimitive]() { throw 'insertion coercion'; }};
  const spliceTarget = {
    get length() { return spliceLength; },
    get 0() { spliceTrace.push('get0'); finalValue = 'live'; return 'first'; },
    get 1() { throw 'deleted index1 Get'; },
    get 2() { throw 'deleted index2 Get'; },
    get 3() { spliceTrace.push('get3'); return finalValue; }
  };
  const start = {[Symbol.toPrimitive](hint) {
    if (hint !== 'number') throw 'start hint';
    spliceTrace.push('start'); spliceLength = 0; return 1;
  }};
  const skip = {[Symbol.toPrimitive](hint) {
    if (hint !== 'number') throw 'deleteCount hint';
    spliceTrace.push('skip'); return 2;
  }};
  check(methods[2].call(observe(spliceTarget, spliceTrace), start, skip, insertion), prototype, ['first', insertion, 'live']);
  if (spliceTrace.join(',') !== 'length,start,skip,0,get0,3,get3') throw 'prefix then insertion then live suffix';
  check(methods[2].call(source), prototype, [3, 1, 2]);
  check(methods[2].call(source, 1), prototype, [3]);
  check(methods[2].call(source, 1, undefined), prototype, [3, 1, 2]);
}
if (borrowedCopies !== 6 || forbiddenGets !== 0) throw 'all borrowed methods without constructor or species';
print('array-by-copy-realms:ok');
262;
