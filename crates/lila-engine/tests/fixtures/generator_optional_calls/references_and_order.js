function check(result, value, done) {
  if (!Object.is(result.value, value) || result.done !== done) throw 'iterator result';
}
const trace = [];
const method = Symbol('method');
const target = {};
const child = {};
let selected;
let chosen = child;
let baseReads = 0;
let keyReads = 0;
let gets = 0;
let tailGets = 0;
let calls = 0;
let proxy;
function original(a, b, c) {
  'use strict';
  trace.push('call');
  calls++;
  if (this !== proxy || a !== 1 || b !== 2 || c !== 3) throw 'original Reference';
  return chosen;
}
Object.defineProperty(target, method, {configurable: true, get() {
  trace.push('get');
  gets++;
  if (this !== proxy) throw 'getter receiver';
  return original;
}});
proxy = new Proxy(target, {get(object, key, receiver) {
  trace.push('proxy');
  if (object !== target || key !== method || receiver !== proxy) throw 'Proxy Reference';
  return Reflect.get(object, key, receiver);
}});
selected = proxy;
function base() { baseReads++; trace.push('base'); return selected; }
const key = {[Symbol.toPrimitive](hint) {
  keyReads++;
  trace.push('key');
  if (hint !== 'string') throw 'key hint';
  return method;
}};
function first() {
  trace.push('first');
  Object.defineProperty(target, method, {configurable: true, value() { throw 'callee reread'; }});
  return 1;
}
function last() { trace.push('last'); return 3; }
Object.defineProperty(child, 'tail', {configurable: true, get() {
  tailGets++;
  trace.push('tail-get');
  return function(value) {
    'use strict';
    trace.push('tail-call');
    if (this !== child || value !== 8) throw 'returned child Reference';
    return function() {
      'use strict';
      trace.push('final-call');
      if (this !== undefined) throw 'Call result receiver reset';
      return 58;
    };
  };
}});
function* run() {
  return base()?.[yield 'key'](first(), yield 'argument', last())?.[yield 'tail-key'](yield 'tail-argument')();
}
const iterator = run();
check(iterator.next(), 'key', false);
selected = {method() { throw 'replacement base'; }};
check(iterator.next(key), 'argument', false);
check(iterator.next(2), 'tail-key', false);
chosen = {tail() { throw 'replacement child'; }};
check(iterator.next('tail'), 'tail-argument', false);
Object.defineProperty(child, 'tail', {configurable: true, value() { throw 'tail callee reread'; }});
check(iterator.next(8), 58, true);
check(iterator.next(), undefined, true);
if (baseReads !== 1 || keyReads !== 1 || gets !== 1 || tailGets !== 1 || calls !== 1) throw 'repeated observations';
if (trace.join(',') !== 'base,key,proxy,get,first,last,call,tail-get,tail-call,final-call') throw 'yielded chain order';

const primitiveKey = Symbol('primitive');
let primitiveGets = 0;
Object.defineProperty(String.prototype, primitiveKey, {configurable: true, get: function() {
  'use strict';
  primitiveGets++;
  if (this !== 'value') throw 'primitive getter receiver';
  return function(value) {
    'use strict';
    if (this !== 'value' || value !== 9) throw 'primitive Call receiver';
    return 19;
  };
}});
function* primitive() { return 'value'[primitiveKey]?.(yield 'primitive-argument'); }
const primitiveIterator = primitive();
check(primitiveIterator.next(), 'primitive-argument', false);
Object.defineProperty(String.prototype, primitiveKey, {configurable: true, value() { throw 'primitive reread'; }});
check(primitiveIterator.next(9), 19, true);
if (primitiveGets !== 1) throw 'primitive Get count';

const spreadTrace = [];
const spreadOwner = {get method() {
  spreadTrace.push('get');
  return function(a, b, c, d) {
    'use strict';
    spreadTrace.push('call');
    if (this !== spreadOwner || a !== 4 || b !== 5 || c !== 6 || d !== 7) throw 'spread snapshot';
    return 47;
  };
}};
const iterable = {[Symbol.iterator]() {
  spreadTrace.push('iterator');
  let index = 0;
  return {next() {
    const step = index++;
    spreadTrace.push('next' + step);
    return {done: step === 2, value: step + 4};
  }};
}};
function tail() { spreadTrace.push('tail'); return 7; }
function* spread() { return spreadOwner.method?.(...iterable, yield 'spread-middle', tail()); }
const spreadIterator = spread();
check(spreadIterator.next(), 'spread-middle', false);
if (spreadTrace.join(',') !== 'get,iterator,next0,next1,next2') throw 'spread must finish before Yield';
check(spreadIterator.next(6), 47, true);
if (spreadTrace.join(',') !== 'get,iterator,next0,next1,next2,tail,call') throw 'spread argument order';
function* indirect() { return eval?.(42, yield 'ignored'); }
const indirectIterator = indirect();
check(indirectIterator.next(), 'ignored', false);
check(indirectIterator.next(0), 42, true);

const groupedTrace = [];
let groupedBaseReads = 0;
let groupedGets = 0;
let groupedKeyReads = 0;
let returnedFunction;
const groupedOwner = {};
const groupedSymbol = Symbol('grouped');
const groupedKey = {[Symbol.toPrimitive](hint) {
  groupedKeyReads++;
  if (hint !== 'string') throw 'grouped key hint';
  groupedTrace.push('key');
  return groupedSymbol;
}};
function groupedOuter(a, b) {
  'use strict';
  groupedTrace.push('outer-call');
  if (this !== undefined || a !== 7 || b !== 8) throw 'grouped outer Value';
  return 78;
}
returnedFunction = groupedOuter;
Object.defineProperty(groupedOwner, groupedSymbol, {configurable: true, get() {
  groupedGets++;
  groupedTrace.push('get');
  return function(a, b) {
    'use strict';
    groupedTrace.push('factory');
    if (this !== groupedOwner || a !== 3 || b !== 4) throw 'grouped inner Reference';
    return returnedFunction;
  };
}});
function groupedBase() { groupedBaseReads++; groupedTrace.push('base'); return groupedOwner; }
function groupedInnerTail() { groupedTrace.push('inner-tail'); return 4; }
function groupedOuterTail() { groupedTrace.push('outer-tail'); return 8; }
function* groupedCalls() {
  return (groupedBase()?.[yield 'grouped-key'](yield 'grouped-inner', groupedInnerTail()))(yield 'grouped-outer', groupedOuterTail());
}
const groupedIterator = groupedCalls();
check(groupedIterator.next(), 'grouped-key', false);
check(groupedIterator.next(groupedKey), 'grouped-inner', false);
Object.defineProperty(groupedOwner, groupedSymbol, {configurable: true, value() { throw 'grouped inner reread'; }});
check(groupedIterator.next(3), 'grouped-outer', false);
returnedFunction = function() { throw 'grouped outer reread'; };
check(groupedIterator.next(7), 78, true);
if (groupedBaseReads !== 1 || groupedGets !== 1 || groupedKeyReads !== 1) throw 'grouped observation counts';
if (groupedTrace.join(',') !== 'base,key,get,inner-tail,factory,outer-tail,outer-call') throw 'grouped callee pin order';

const groupedSpreadTrace = [];
const groupedSpreadOwner = {make(value) {
  'use strict';
  groupedSpreadTrace.push('factory');
  if (this !== groupedSpreadOwner || value !== 1) throw 'spread factory Reference';
  return function(a, b, c) {
    'use strict';
    groupedSpreadTrace.push('outer');
    if (this !== undefined || a !== 4 || b !== 5 || c !== 6) throw 'grouped outer spread';
    return 46;
  };
}};
const groupedIterable = {[Symbol.iterator]() {
  groupedSpreadTrace.push('iterator');
  let index = 0;
  return {next() {
    const current = index++;
    groupedSpreadTrace.push('next' + current);
    return {done: current === 2, value: current + 4};
  }};
}};
function* groupedSpread() { return (groupedSpreadOwner?.make(yield 'spread-factory'))(...groupedIterable, yield 'spread-outer'); }
const groupedSpreadIterator = groupedSpread();
check(groupedSpreadIterator.next(), 'spread-factory', false);
check(groupedSpreadIterator.next(1), 'spread-outer', false);
if (groupedSpreadTrace.join(',') !== 'factory,iterator,next0,next1,next2') throw 'grouped spread must finish before outer Yield';
check(groupedSpreadIterator.next(6), 46, true);
if (groupedSpreadTrace.join(',') !== 'factory,iterator,next0,next1,next2,outer') throw 'grouped spread Call order';

const groupedPrimitiveSymbol = Symbol('grouped-primitive');
Object.defineProperty(String.prototype, groupedPrimitiveSymbol, {configurable: true, value: function(value) {
  'use strict';
  if (this !== 'raw' || value !== 2) throw 'grouped primitive inner receiver';
  return function(argument) {
    'use strict';
    if (this !== undefined || argument !== 3) throw 'grouped primitive outer receiver';
    return 23;
  };
}});
function* groupedPrimitive() { return ('raw'?.[groupedPrimitiveSymbol](yield 'primitive-inner'))(yield 'primitive-outer'); }
const groupedPrimitiveIterator = groupedPrimitive();
check(groupedPrimitiveIterator.next(), 'primitive-inner', false);
check(groupedPrimitiveIterator.next(2), 'primitive-outer', false);
check(groupedPrimitiveIterator.next(3), 23, true);

let firstTemplate;
let groupedTags = 0;
const groupedTagOwner = {make(value) {
  'use strict';
  if (this !== groupedTagOwner || value !== 1) throw 'grouped tag factory Reference';
  return function(template, substitution) {
    'use strict';
    groupedTags++;
    if (this !== undefined || substitution !== 6) throw 'returned tag Value receiver';
    if (!Object.isFrozen(template) || !Object.isFrozen(template.raw) || template.length !== 2 || template[0] !== 'head' || template[1] !== 'tail' || template.raw[0] !== 'head' || template.raw[1] !== 'tail') throw 'grouped template payload';
    if (firstTemplate === undefined) firstTemplate = template;
    else if (firstTemplate !== template) throw 'grouped original template site';
    return 16;
  };
}};
function* groupedTag() { return (groupedTagOwner?.make(yield 'tag-factory'))`head${yield 'tag-substitution'}tail`; }
const groupedTagFirst = groupedTag();
check(groupedTagFirst.next(), 'tag-factory', false);
check(groupedTagFirst.next(1), 'tag-substitution', false);
check(groupedTagFirst.next(6), 16, true);
const groupedTagSecond = groupedTag();
check(groupedTagSecond.next(), 'tag-factory', false);
check(groupedTagSecond.next(1), 'tag-substitution', false);
check(groupedTagSecond.next(6), 16, true);
if (groupedTags !== 2) throw 'grouped tag Call count';
const groupedPropertyTrace = [];
const groupedPropertySymbol = Symbol('grouped-property');
let groupedPropertySelected;
let groupedPropertyBaseReads = 0;
let groupedPropertyChildGets = 0;
let groupedPropertyTerminalGets = 0;
let groupedPropertyKeyReads = 0;
let groupedPropertyCalls = 0;
const groupedPropertyChildTarget = {};
let groupedPropertyChildProxy;
Object.defineProperty(groupedPropertyChildTarget, groupedPropertySymbol, {configurable: true, get() {
  groupedPropertyTerminalGets++;
  groupedPropertyTrace.push('get');
  if (this !== groupedPropertyChildProxy) throw 'grouped Property getter receiver';
  return function(a, b, c, d, e) {
    'use strict';
    groupedPropertyCalls++;
    groupedPropertyTrace.push('call');
    if (this !== groupedPropertyChildProxy || a !== 1 || b !== 2 || c !== 3 || d !== 4 || e !== 5) throw 'grouped Property retained Reference';
    return 65;
  };
}});
groupedPropertyChildProxy = new Proxy(groupedPropertyChildTarget, {get(object, property, receiver) {
  groupedPropertyTrace.push('proxy');
  if (object !== groupedPropertyChildTarget || property !== groupedPropertySymbol || receiver !== groupedPropertyChildProxy) throw 'grouped Property Proxy Reference';
  return Reflect.get(object, property, receiver);
}});
const groupedPropertyRoot = {get child() {
  groupedPropertyChildGets++;
  groupedPropertyTrace.push('child-get');
  return groupedPropertyChildProxy;
}};
groupedPropertySelected = groupedPropertyRoot;
function groupedPropertyBase() {
  groupedPropertyBaseReads++;
  groupedPropertyTrace.push('base');
  return groupedPropertySelected;
}
const groupedPropertyKey = {[Symbol.toPrimitive](hint) {
  groupedPropertyKeyReads++;
  groupedPropertyTrace.push('key');
  if (hint !== 'string') throw 'grouped Property key hint';
  return groupedPropertySymbol;
}};
function groupedPropertyFirst() {
  groupedPropertyTrace.push('first');
  Object.defineProperty(groupedPropertyChildTarget, groupedPropertySymbol, {configurable: true, value() { throw 'grouped Property callee reread'; }});
  return 1;
}
const groupedPropertyIterable = {[Symbol.iterator]() {
  groupedPropertyTrace.push('iterator');
  let index = 0;
  return {next() {
    const current = index++;
    groupedPropertyTrace.push('next' + current);
    return {done: current === 2, value: current + 2};
  }};
}};
function groupedPropertyLast() { groupedPropertyTrace.push('last'); return 5; }
function* groupedPropertyCall() {
  return (groupedPropertyBase()?.[yield 'grouped-property-child']?.[yield 'grouped-property-method'])(groupedPropertyFirst(), ...groupedPropertyIterable, yield 'grouped-property-argument', groupedPropertyLast());
}
const groupedPropertyIterator = groupedPropertyCall();
check(groupedPropertyIterator.next(), 'grouped-property-child', false);
groupedPropertySelected = {child: {method() { throw 'grouped Property base reread'; }}};
check(groupedPropertyIterator.next('child'), 'grouped-property-method', false);
Object.defineProperty(groupedPropertyRoot, 'child', {value: {method() { throw 'grouped Property child reread'; }}});
check(groupedPropertyIterator.next(groupedPropertyKey), 'grouped-property-argument', false);
if (groupedPropertyTrace.join(',') !== 'base,child-get,key,proxy,get,first,iterator,next0,next1,next2') throw 'grouped Property capture before outer operands';
check(groupedPropertyIterator.next(4), 65, true);
check(groupedPropertyIterator.next(), undefined, true);
if (groupedPropertyBaseReads !== 1 || groupedPropertyChildGets !== 1 || groupedPropertyTerminalGets !== 1 || groupedPropertyKeyReads !== 1 || groupedPropertyCalls !== 1) throw 'grouped Property observation counts';
if (groupedPropertyTrace.join(',') !== 'base,child-get,key,proxy,get,first,iterator,next0,next1,next2,last,call') throw 'grouped Property outer argument order';

const groupedPropertyPrimitiveSymbol = Symbol('grouped-property-primitive');
let groupedPropertyPrimitiveGets = 0;
Object.defineProperty(String.prototype, groupedPropertyPrimitiveSymbol, {configurable: true, get: function() {
  'use strict';
  groupedPropertyPrimitiveGets++;
  if (this !== 'raw') throw 'grouped Property primitive Get receiver';
  return function(argument) {
    'use strict';
    if (this !== 'raw' || argument !== 7) throw 'grouped Property primitive Call receiver';
    return 27;
  };
}});
function* groupedPropertyPrimitive() { return ('raw'?.[yield 'grouped-property-primitive-key'])(yield 'grouped-property-primitive-argument'); }
const groupedPropertyPrimitiveIterator = groupedPropertyPrimitive();
check(groupedPropertyPrimitiveIterator.next(), 'grouped-property-primitive-key', false);
check(groupedPropertyPrimitiveIterator.next(groupedPropertyPrimitiveSymbol), 'grouped-property-primitive-argument', false);
Object.defineProperty(String.prototype, groupedPropertyPrimitiveSymbol, {configurable: true, value() { throw 'grouped Property primitive reread'; }});
check(groupedPropertyPrimitiveIterator.next(7), 27, true);
if (groupedPropertyPrimitiveGets !== 1) throw 'grouped Property primitive Get count';

let groupedPropertyTemplate;
let groupedPropertyTagCalls = 0;
let groupedPropertyTagGets = 0;
const groupedPropertyTagOwner = {};
const groupedPropertyTagSymbol = Symbol('grouped-property-tag');
function groupedPropertyTagGetter() {
  groupedPropertyTagGets++;
  if (this !== groupedPropertyTagOwner) throw 'grouped Property tag Get receiver';
  return function(template, substitution) {
    'use strict';
    groupedPropertyTagCalls++;
    if (this !== groupedPropertyTagOwner || substitution !== 8) throw 'grouped Property tag Reference';
    if (!Object.isFrozen(template) || !Object.isFrozen(template.raw) || template.length !== 2 || template[0] !== 'head' || template[1] !== 'tail' || template.raw[0] !== 'head' || template.raw[1] !== 'tail') throw 'grouped Property template payload';
    if (groupedPropertyTemplate === undefined) groupedPropertyTemplate = template;
    else if (groupedPropertyTemplate !== template) throw 'grouped Property original template site';
    return 28;
  };
}
Object.defineProperty(groupedPropertyTagOwner, groupedPropertyTagSymbol, {configurable: true, get: groupedPropertyTagGetter});
function* groupedPropertyTag() { return (groupedPropertyTagOwner?.[yield 'grouped-property-tag-key'])`head${yield 'grouped-property-tag-substitution'}tail`; }
const groupedPropertyTagFirst = groupedPropertyTag();
check(groupedPropertyTagFirst.next(), 'grouped-property-tag-key', false);
check(groupedPropertyTagFirst.next(groupedPropertyTagSymbol), 'grouped-property-tag-substitution', false);
Object.defineProperty(groupedPropertyTagOwner, groupedPropertyTagSymbol, {configurable: true, value() { throw 'grouped Property tag reread'; }});
check(groupedPropertyTagFirst.next(8), 28, true);
Object.defineProperty(groupedPropertyTagOwner, groupedPropertyTagSymbol, {configurable: true, get: groupedPropertyTagGetter});
const groupedPropertyTagSecond = groupedPropertyTag();
check(groupedPropertyTagSecond.next(), 'grouped-property-tag-key', false);
check(groupedPropertyTagSecond.next(groupedPropertyTagSymbol), 'grouped-property-tag-substitution', false);
check(groupedPropertyTagSecond.next(8), 28, true);
if (groupedPropertyTagCalls !== 2 || groupedPropertyTagGets !== 2) throw 'grouped Property tag observation counts';
print('generator-optional-references:ok');
262;
