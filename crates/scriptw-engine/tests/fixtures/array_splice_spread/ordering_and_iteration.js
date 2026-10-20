const SavedObject = Object;
const SavedArray = Array;
const splice = Array.prototype.splice;
const defineProperty = Object.defineProperty;
const hasOwn = Object.prototype.hasOwnProperty;

function checkArray(actual, expected) {
  if (actual.length !== expected.length) throw 'array length';
  for (let index = 0; index < expected.length; ++index) {
    if (actual[index] !== expected[index]) throw 'array element';
  }
}
function dynamic(value) { return value; }

const literal = [1, 2];
const literalDeleted = literal.splice(0, 0, ...Object.keys({ alpha: 1, beta: 2 }));
checkArray(literalDeleted, []);
checkArray(literal, ['alpha', 'beta', 1, 2]);

const dynamicSource = dynamic([10, 20, 30, 40]);
const alias = dynamicSource;
let start = 1;
let deleteCount = 2;
const dynamicDeleted = alias.splice(start, deleteCount, ...Object.keys({ a: 1, b: 2 }));
if (alias !== dynamicSource) throw 'alias receiver';
checkArray(dynamicDeleted, [20, 30]);
checkArray(dynamicSource, [10, 'a', 'b', 40]);

const trace = [];
const capturedSource = [10, 20, 30];
const alternativeSource = [100];
let selectedSource = capturedSource;
let wrongCalls = 0;
function wrongSplice() { ++wrongCalls; throw 'replaced splice'; }
defineProperty(capturedSource, 'splice', {
  configurable: true,
  get() { trace.push('callee'); return splice; }
});
function receiver() { trace.push('receiver'); return selectedSource; }
function startArgument() {
  trace.push('start.arg');
  return { [Symbol.toPrimitive](hint) {
    if (hint !== 'number') throw 'start hint';
    trace.push('start.coerce');
    return 1;
  } };
}
function deleteArgument() {
  trace.push('delete.arg');
  return { [Symbol.toPrimitive](hint) {
    if (hint !== 'number') throw 'delete hint';
    trace.push('delete.coerce');
    return 1;
  } };
}
const keyToken = {};
function keyArgument() { trace.push('keys.arg'); return keyToken; }
const returnedArray = [999, 888];
defineProperty(returnedArray, '0', { get() { throw 'numeric spread snapshot'; } });
defineProperty(returnedArray, '1', { get() { throw 'numeric spread snapshot'; } });
let step = 0;
const iterator = {};
function nextTarget() { throw 'next target bypass'; }
const nextMethod = new Proxy(nextTarget, {
  apply(target, thisArgument, argumentsList) {
    if (target !== nextTarget || thisArgument !== iterator || argumentsList.length !== 0) {
      throw 'next apply receiver';
    }
    trace.push('next.apply');
    const position = step++;
    if (position > 2) throw 'extra spread step';
    if (position === 0) defineProperty(iterator, 'next', { value: wrongSplice });
    return {
      get done() { trace.push('done' + position); return position === 2; },
      get value() {
        if (position === 2) throw 'terminal value';
        trace.push('value' + position);
        return position === 0 ? 'x' : 'y';
      }
    };
  }
});
defineProperty(iterator, 'next', {
  configurable: true,
  get() { trace.push('next.get'); return nextMethod; }
});
defineProperty(iterator, 'return', { get() { throw 'normal spread close'; } });
function iteratorTarget() { throw 'iterator target bypass'; }
const iteratorMethod = new Proxy(iteratorTarget, {
  apply(target, thisArgument, argumentsList) {
    if (target !== iteratorTarget || thisArgument !== returnedArray || argumentsList.length !== 0) {
      throw 'iterator apply receiver';
    }
    trace.push('iterator.apply');
    return iterator;
  }
});
defineProperty(returnedArray, Symbol.iterator, {
  get() {
    trace.push('iterator.get');
    capturedSource[1] = 200;
    defineProperty(capturedSource, 'splice', { configurable: true, value: wrongSplice });
    return iteratorMethod;
  }
});
const removedObject = {};
function Species(length) {
  trace.push('construct:' + length);
  if (length !== 1 || capturedSource.length !== 3 || capturedSource[0] !== 99 ||
      capturedSource[1] !== 200 || capturedSource[2] !== 30) throw 'species before mutation';
  return removedObject;
}
const speciesCarrier = {
  get [Symbol.species]() { trace.push('species'); return Species; }
};
defineProperty(capturedSource, 'constructor', {
  get() { trace.push('constructor'); return speciesCarrier; }
});
const keysObject = {
  get keys() {
    trace.push('keys.get');
    defineProperty(capturedSource, 'splice', { configurable: true, value: wrongSplice });
    selectedSource = alternativeSource;
    return function (argument) {
      if (this !== keysObject || argument !== keyToken) throw 'keys receiver';
      trace.push('keys.call');
      capturedSource[0] = 99;
      return returnedArray;
    };
  }
};
globalThis.Object = keysObject;
const capturedDeleted = receiver().splice(startArgument(), deleteArgument(), ...Object.keys(keyArgument()));
globalThis.Object = SavedObject;
const expectedTrace = 'receiver,callee,start.arg,delete.arg,keys.get,keys.arg,keys.call,' +
  'iterator.get,iterator.apply,next.get,next.apply,done0,value0,next.apply,done1,value1,' +
  'next.apply,done2,start.coerce,delete.coerce,constructor,species,construct:1';
if (trace.join(',') !== expectedTrace) throw 'captured reference trace';
if (wrongCalls !== 0 || capturedDeleted !== removedObject || capturedDeleted.length !== 1 ||
    capturedDeleted[0] !== 200 || selectedSource !== alternativeSource) throw 'captured reference result';
checkArray(capturedSource, [99, 'x', 'y', 30]);
checkArray(alternativeSource, [100]);

const zeroTrace = [];
const zeroSource = [1, 2];
const zeroResult = {};
function ZeroSpecies(length) {
  zeroTrace.push('construct:' + length);
  if (length !== 0 || zeroSource.length !== 2 || zeroSource[0] !== 1 || zeroSource[1] !== 2) {
    throw 'zero-delete species order';
  }
  return zeroResult;
}
defineProperty(zeroSource, 'constructor', {
  get() {
    zeroTrace.push('constructor');
    return { get [Symbol.species]() { zeroTrace.push('species'); return ZeroSpecies; } };
  }
});
const zeroDeleted = zeroSource.splice(0, 0, ...Object.keys({ before: 1 }));
if (zeroDeleted !== zeroResult || zeroResult.length !== 0 ||
    zeroTrace.join(',') !== 'constructor,species,construct:0') throw 'zero-delete species result';
checkArray(zeroSource, ['before', 1, 2]);

{
  const Object = {
    keys() {
      let emitted = false;
      return { [Symbol.iterator]() {
        return { next() {
          if (emitted) return { done: true };
          emitted = true;
          return { done: false, value: 42 };
        } };
      } };
    }
  };
  const shadowed = [0];
  const shadowedDeleted = shadowed.splice(0, 0, ...Object.keys({}));
  checkArray(shadowedDeleted, []);
  checkArray(shadowed, [42, 0]);
}

const sparse = [1, , 3, , 5];
const inherited = SavedObject.create(SavedArray.prototype);
defineProperty(inherited, '1', { configurable: true, writable: true, value: 2 });
SavedObject.setPrototypeOf(sparse, inherited);
const propertyTrace = [];
const proxy = new Proxy(sparse, {
  get(target, key, receiver) {
    propertyTrace.push('get:' + key);
    if (key === 'constructor') return undefined;
    return Reflect.get(target, key, receiver);
  },
  has(target, key) { propertyTrace.push('has:' + key); return Reflect.has(target, key); },
  set(target, key, value, receiver) {
    propertyTrace.push('set:' + key);
    return Reflect.set(target, key, value, receiver);
  },
  deleteProperty(target, key) {
    propertyTrace.push('delete:' + key);
    return Reflect.deleteProperty(target, key);
  }
});
const sparseDeleted = proxy.splice(1, 3, ...Object.keys({ insert: 1 }));
if (propertyTrace.join(',') !== 'get:splice,get:length,get:constructor,has:1,get:1,has:2,get:2,' +
    'has:3,has:4,get:4,set:2,delete:4,delete:3,set:1,set:length') throw 'splice property trace';
if (sparseDeleted.length !== 3 || sparseDeleted[0] !== 2 || sparseDeleted[1] !== 3 ||
    hasOwn.call(sparseDeleted, '2') || !hasOwn.call(sparse, '1')) throw 'splice inherited holes';
checkArray(sparse, [1, 'insert', 5]);

print('array-splice-spread-order:ok');
262;
