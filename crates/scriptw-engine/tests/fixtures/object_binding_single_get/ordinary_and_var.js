function check(condition, name) { if (!condition) throw name; }
const trace = [];
let gets = 0, defaults = 0, sources = 0;
function fallback() { defaults++; trace.push('default'); return 19; }
function source(value) {
  sources++; trace.push('source');
  return {get item() { gets++; trace.push('get'); return value; }};
}
{
  let {item = fallback()} = source(7);
  check(item === 7 && gets === 1 && defaults === 0 && sources === 1,
    'nonundefined lexical getter is retained once');
}
{
  const {item = fallback()} = source(undefined);
  check(item === 19 && gets === 2 && defaults === 1 && sources === 2,
    'undefined lexical getter runs one default');
}
function varOwner() {
  var {item = fallback()} = source(null);
  check(item === null, 'null does not select default');
  var {item = fallback()} = source(undefined);
  return item;
}
check(varOwner() === 19 && gets === 4 && defaults === 2 && sources === 4,
  'var retains null and one selected default');
check(trace.join(',') === 'source,get,source,get,default,source,get,source,get,default',
  'all RHS/Get/default phases execute in order');

let proxyGets = 0;
const target = {item: 23};
const proxy = new Proxy(target, {get(object, key, rawThis) {
  check(object === target && key === 'item' && rawThis === proxy, 'original Proxy Get receiver');
  proxyGets++; return object[key];
}});
const {item: proxyValue = fallback()} = proxy;
check(proxyValue === 23 && proxyGets === 1 && defaults === 2, 'Proxy default decision uses acquired value');

const order = [];
const pair = {get first() { order.push('first'); return 3; }, get second() { order.push('second'); return undefined; }};
const {first, second = (order.push('default.second'), first + 4)} = pair;
check(first === 3 && second === 7 && order.join(',') === 'first,second,default.second',
  'earlier initialized sibling feeds later default');
const define = Object.defineProperty;
const descriptor = Object.getOwnPropertyDescriptor;
const primitiveKey = 'bindingPrimitiveReceiver';
const oldString = descriptor(String.prototype, primitiveKey);
const oldNumber = descriptor(Number.prototype, primitiveKey);
let primitiveStringGets = 0, primitiveNumberGets = 0, primitiveDefaults = 0;
try {
  define(String.prototype, primitiveKey, {configurable: true, get() {
    'use strict';
    check(typeof this === 'string' && this === 'source', 'GetV retains primitive String receiver');
    primitiveStringGets++; return 11;
  }});
  define(Number.prototype, primitiveKey, {configurable: true, get() {
    'use strict';
    check(typeof this === 'number' && this === 5, 'GetV retains primitive Number receiver');
    primitiveNumberGets++; return undefined;
  }});
  const {bindingPrimitiveReceiver: stringValue = (primitiveDefaults++, 19)} = 'source';
  var {bindingPrimitiveReceiver: numberValue = (primitiveDefaults++, 17)} = 5;
  let assigned;
  ({bindingPrimitiveReceiver: assigned = (primitiveDefaults++, 23)} = 'source');
  check(stringValue === 11 && numberValue === 17 && assigned === 11 &&
    primitiveStringGets === 2 && primitiveNumberGets === 1 && primitiveDefaults === 1,
    'primitive binding/assignment GetV observes one value and selected default');
} finally {
  if (oldString === undefined) delete String.prototype[primitiveKey];
  else define(String.prototype, primitiveKey, oldString);
  if (oldNumber === undefined) delete Number.prototype[primitiveKey];
  else define(Number.prototype, primitiveKey, oldNumber);
}
const empty = {};
const {} = empty;
var {} = 7;
print('object-binding-ordinary:ok');
262;
