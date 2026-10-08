const trace = [];
function check(result, value, done) {
  if (!Object.is(result.value, value) || result.done !== done) throw 'iterator result';
}
const key = Symbol('original');
const leaf = {get value() { trace.push('leaf'); return 17; }};
const original = {[key]: {get child() { trace.push('child'); return leaf; }}};
const proxy = new Proxy(original, {get(target, name, receiver) {
  if (target !== original || name !== key || receiver !== proxy) throw 'optional Get Reference';
  trace.push('get');
  return Reflect.get(target, name, receiver);
}});
let source = proxy;
let baseReads = 0;
const holder = {get base() { baseReads++; trace.push('base'); return source; }};
function operand() { trace.push('operand'); return 'key-token'; }
function* read() { return holder.base?.[(yield operand())].child.value; }
const iterator = read();
check(iterator.next(), 'key-token', false);
trace.push('caller');
source = {get wrong() { throw 'replacement base'; }};
let coercions = 0;
const receivedKey = {[Symbol.toPrimitive](hint) {
  coercions++;
  if (hint !== 'string') throw 'key hint';
  trace.push('coerce');
  return key;
}};
check(iterator.next(receivedKey), 17, true);
if (baseReads !== 1 || coercions !== 1 || trace.join(',') !== 'base,operand,caller,coerce,get,child,leaf') throw 'base retention or eager suffix order';

let skippedCalls = 0;
function skipped() { skippedCalls++; throw 'shorted optional operand'; }
function* shorted(base) { var result = base?.[(yield skipped())].missing.deep; return result; }
check(shorted(null).next(), undefined, true);
check(shorted(undefined).next(), undefined, true);
if (skippedCalls !== 0) throw 'whole suffix was not shorted';
function* stringValue() { const result = 'ab'?.[(yield 'index')]; return result; }
const string = stringValue();
check(string.next(), 'index', false);
check(string.next(1), 'b', true);
let primitiveReads = 0;
Object.defineProperty(Number.prototype, 'generatorOptionalValue', {get: function() {
  'use strict';
  primitiveReads++;
  if (this !== 0) throw 'primitive Get receiver';
  return 19;
}, configurable: true});
function* primitiveValue() { return 0?.[(yield 'number-key')]; }
const primitive = primitiveValue();
check(primitive.next(), 'number-key', false);
check(primitive.next('generatorOptionalValue'), 19, true);
delete Number.prototype.generatorOptionalValue;
if (primitiveReads !== 1) throw 'falsy nonnullish base was skipped';

const child = {method(value) { if (this !== child) throw 'outer ordinary Reference'; return value + 1; }};
const table = {entry: child};
function* outerCall() { return (table?.[(yield 'entry-key')]).method(40); }
const outer = outerCall();
check(outer.next(), 'entry-key', false);
check(outer.next('entry'), 41, true);
print('generator-properties:ok');
