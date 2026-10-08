const trace = [];
const symbol = Symbol('method');
const target = {};
const replacement = {[symbol]() { throw 'replacement callee'; }};
let proxy;
let current;
let baseCalls = 0;
let keyConversions = 0;
let proxyGets = 0;
let getterCalls = 0;
function original(a, b, c) {
  'use strict';
  trace.push('call');
  if (this !== proxy || a !== 1 || b !== 2 || c !== 3) throw 'original Reference or arguments';
  return 31;
}
Object.defineProperty(target, symbol, {configurable: true, get() {
  if (this !== proxy) throw 'accessor receiver';
  getterCalls++;
  trace.push('getter');
  return original;
}});
proxy = new Proxy(target, {get(object, key, receiver) {
  if (object !== target || key !== symbol || receiver !== proxy) throw 'Proxy Reference';
  proxyGets++;
  trace.push('proxy-get');
  return Reflect.get(object, key, receiver);
}});
current = proxy;
function base() { baseCalls++; trace.push('base'); return current; }
const keyObject = {[Symbol.toPrimitive](hint) {
  if (hint !== 'string') throw 'key hint';
  keyConversions++;
  trace.push('key');
  return symbol;
}};
const keyThenable = {get then() {
  trace.push('key-then');
  current = replacement;
  return resolve => { trace.push('key-resolve'); resolve(keyObject); };
}};
function keySource() { trace.push('key-source'); return keyThenable; }
function firstArgument() {
  trace.push('arg-a');
  Object.defineProperty(target, symbol, {configurable: true, value() { throw 'reread callee'; }});
  return 1;
}
const argumentThenable = {get then() {
  trace.push('arg-then');
  current = replacement;
  return resolve => { trace.push('arg-resolve'); resolve(2); };
}};
function lastArgument() { trace.push('arg-c'); return 3; }
async function run() {
  const result = (base()?.[await keySource()])(firstArgument(), await argumentThenable, lastArgument());
  if (result !== 31 || baseCalls !== 1 || keyConversions !== 1 || proxyGets !== 1 || getterCalls !== 1 || current !== replacement) throw 'repeated callee observation';
  if (trace.join(',') !== 'base,key-source,key-then,caller,key-resolve,key,proxy-get,getter,arg-a,arg-then,arg-resolve,arg-c,call') throw 'callee and argument order';
  const firstTraceLength = trace.length;
  const primitiveSymbol = Symbol('primitive');
  let primitiveGets = 0;
  Object.defineProperty(String.prototype, primitiveSymbol, {configurable: true, get: function() {
    'use strict';
    if (this !== 'value') throw 'primitive accessor receiver';
    primitiveGets++;
    trace.push('primitive-get');
    return function(value) {
      'use strict';
      trace.push('primitive-call');
      if (this !== 'value' || value !== 9) throw 'boxed primitive receiver';
      return 19;
    };
  }});
  const primitiveKey = {[Symbol.toPrimitive](hint) {
    if (hint !== 'string') throw 'primitive key hint';
    trace.push('primitive-key');
    return primitiveSymbol;
  }};
  function primitiveArgument() {
    trace.push('primitive-arg');
    Object.defineProperty(String.prototype, primitiveSymbol, {configurable: true, value() { throw 'primitive reread'; }});
    return 9;
  }
  if (('value'?.[await primitiveKey])(primitiveArgument()) !== 19 || primitiveGets !== 1) throw 'primitive Reference retention';
  const spreadTarget = {get method() {
    trace.push('spread-get');
    return function(a, b, c, d) {
      'use strict';
      trace.push('spread-call');
      if (this !== spreadTarget || a !== 4 || b !== 5 || c !== 6 || d !== 7) throw 'spread values or receiver';
      return 47;
    };
  }};
  const iterable = {[Symbol.iterator]() {
    trace.push('iterator');
    let index = 0;
    return {next() {
      const step = index++;
      trace.push('next' + step);
      return {get done() { trace.push('done' + step); return step === 2; }, get value() { trace.push('value' + step); return step + 4; }};
    }, return() { throw 'ordinary argument spread must not close'; }};
  }};
  function afterSpread() { trace.push('after-spread'); return 6; }
  function spreadTail() { trace.push('spread-tail'); return 7; }
  if ((spreadTarget?.[await 'method'])(...iterable, await afterSpread(), spreadTail()) !== 47) throw 'spread call result';
  if (trace.slice(firstTraceLength).join(',') !== 'primitive-key,primitive-get,primitive-arg,primitive-call,spread-get,iterator,next0,done0,value0,next1,done1,value1,next2,done2,after-spread,spread-tail,spread-call') throw 'primitive and spread order';
  const multiTraceLength = trace.length;
  let chainBaseCalls = 0;
  let firstConversions = 0;
  let lastConversions = 0;
  let middleGets = 0;
  let finalGets = 0;
  const child = {};
  const parent = {};
  const replacementChild = {method() { throw 'replacement child callee'; }};
  let selectedParent = parent;
  Object.defineProperty(parent, 'child', {configurable: true, get() {
    if (this !== parent) throw 'intermediate receiver';
    middleGets++;
    trace.push('middle-get');
    return child;
  }});
  Object.defineProperty(child, 'method', {configurable: true, get() {
    if (this !== child) throw 'terminal getter receiver';
    finalGets++;
    trace.push('final-get');
    return function(value) {
      'use strict';
      trace.push('multi-call');
      if (this !== child || value !== 9) throw 'terminal Reference after intermediate await';
      return 59;
    };
  }});
  function chainBase() { chainBaseCalls++; trace.push('multi-base'); return selectedParent; }
  const firstKey = {[Symbol.toPrimitive](hint) {
    if (hint !== 'string') throw 'first key hint';
    firstConversions++;
    trace.push('first-key');
    return 'child';
  }};
  const lastKey = {[Symbol.toPrimitive](hint) {
    if (hint !== 'string') throw 'last key hint';
    lastConversions++;
    trace.push('last-key');
    return 'method';
  }};
  function firstKeySource() { trace.push('first-source'); return firstKey; }
  const lastThenable = {get then() {
    trace.push('last-then');
    selectedParent = {child: replacementChild};
    Object.defineProperty(parent, 'child', {configurable: true, value: replacementChild});
    return resolve => { trace.push('last-resolve'); resolve(lastKey); };
  }};
  function lastKeySource() { trace.push('last-source'); return lastThenable; }
  function chainArgument() {
    trace.push('multi-arg');
    Object.defineProperty(child, 'method', {configurable: true, value() { throw 'terminal callee reread'; }});
    return 9;
  }
  if ((chainBase()?.[await firstKeySource()]?.[await lastKeySource()])(chainArgument()) !== 59) throw 'nested chain result';
  if (chainBaseCalls !== 1 || firstConversions !== 1 || lastConversions !== 1 || middleGets !== 1 || finalGets !== 1 || parent.child !== replacementChild) throw 'nested chain repeated observation';
  let skippedSuffixKeys = 0;
  const shortParent = {get child() { trace.push('short-get'); return null; }};
  function shortFirstKey() { trace.push('short-key'); return 'child'; }
  function skippedSuffixKey() { skippedSuffixKeys++; throw 'shorted suffix awaited key'; }
  function shortArgument() { trace.push('short-arg'); return 1; }
  try { (shortParent?.[await shortFirstKey()]?.[await skippedSuffixKey()])(shortArgument()); }
  catch (error) {
    if (Object.getPrototypeOf(error) !== TypeError.prototype) throw 'nested short grouped call TypeError';
    trace.push('short-thrown');
  }
  if (skippedSuffixKeys !== 0 || trace.slice(multiTraceLength).join(',') !== 'multi-base,first-source,first-key,middle-get,last-source,last-then,last-resolve,last-key,final-get,multi-arg,multi-call,short-key,short-get,short-arg,short-thrown') throw 'intermediate and terminal capture order';

  const callTrace = [];
  let valueGets = 0;
  let valueCalls = 0;
  let selectedFactory;
  const valueFactory = {};
  function returnedValue(value) {
    'use strict';
    callTrace.push('returned');
    valueCalls++;
    if (this !== undefined || value !== 12) throw 'terminal Call is a Value';
    return 71;
  }
  Object.defineProperty(valueFactory, 'make', {configurable: true, get() {
    if (this !== valueFactory) throw 'inner factory getter receiver';
    callTrace.push('get');
    valueGets++;
    return function(value) {
      'use strict';
      callTrace.push('make');
      if (this !== valueFactory || value !== 11) throw 'inner Call Reference';
      return returnedValue;
    };
  }});
  selectedFactory = valueFactory;
  function valueBase() { callTrace.push('base'); return selectedFactory; }
  function innerValue() { callTrace.push('inner'); return 11; }
  const outerValue = {get then() {
    callTrace.push('outer-then');
    selectedFactory = {make() { throw 'replacement factory'; }};
    Object.defineProperty(valueFactory, 'make', {value() { throw 'reread factory'; }});
    return resolve => { callTrace.push('outer-resolve'); resolve(12); };
  }};
  if ((valueBase()?.make(await innerValue()))(await outerValue) !== 71 || valueGets !== 1 || valueCalls !== 1) throw 'completed Call value retained';
  if (callTrace.join(',') !== 'base,get,inner,make,outer-then,outer-resolve,returned') throw 'terminal Call and outer argument order';

  let nestedGets = 0;
  let nestedCalls = 0;
  const nested = {get make() {
    nestedGets++;
    return function() {
      'use strict';
      nestedCalls++;
      if (this !== nested) throw 'target-only inner grouped Reference';
      return function(value) {
        'use strict';
        if (this !== undefined || value !== 13) throw 'target-only terminal Call receiver reset';
        return 73;
      };
    };
  }};
  if (((nested?.[await 'make'])?.())(await 13) !== 73 || nestedGets !== 1 || nestedCalls !== 1) throw 'target-only grouped terminal Call';

  const directAwaitFactory = {make() {
    'use strict';
    if (this !== directAwaitFactory) throw 'direct awaited base inner receiver';
    return function(value) {
      'use strict';
      if (this !== undefined || value !== 23) throw 'direct awaited base outer Value receiver';
      return 97;
    };
  }};
  if (((await directAwaitFactory)?.make())(await 23) !== 97) throw 'direct awaited method Call value';
  const directAwaitFunction = function() {
    'use strict';
    if (this !== undefined) throw 'direct awaited function inner receiver';
    return function(value) {
      'use strict';
      if (this !== undefined || value !== 29) throw 'direct awaited function outer receiver';
      return 101;
    };
  };
  if (((await directAwaitFunction)?.())(await 29) !== 101) throw 'direct awaited first Call value';

  const primitiveCall = Symbol('primitive-call-value');
  let primitiveCallGets = 0;
  Object.defineProperty(String.prototype, primitiveCall, {configurable: true, get: function() {
    'use strict';
    primitiveCallGets++;
    if (this !== 'raw') throw 'inner primitive getter';
    return function() {
      'use strict';
      if (this !== 'raw') throw 'inner primitive Call';
      return function(value) {
        'use strict';
        if (this !== undefined || value !== 17) throw 'outer primitive Call value';
        return 79;
      };
    };
  }});
  if (('raw'?.[await primitiveCall]())(17) !== 79 || primitiveCallGets !== 1) throw 'primitive Call value ownership';

  const spreadTrace = [];
  const spreadFactory = {make(value) {
    if (this !== spreadFactory || value !== 1) throw 'spread factory receiver';
    spreadTrace.push('make');
    return function(a, b, c) {
      'use strict';
      spreadTrace.push('call');
      if (this !== undefined || a !== 1 || b !== 2 || c !== 3) throw 'outer spread Value call';
      return 83;
    };
  }};
  const outerIterable = {[Symbol.iterator]() {
    spreadTrace.push('iterator');
    let step = 0;
    return {next() {
      const done = step++ !== 0;
      spreadTrace.push(done ? 'done' : 'value');
      return {done, value: 1};
    }, return() { throw 'completed outer spread closed'; }};
  }};
  function outerTail() { spreadTrace.push('tail'); return 2; }
  if ((spreadFactory?.make(await 1))(...outerIterable, await outerTail(), 3) !== 83) throw 'outer spread result';
  if (spreadTrace.join(',') !== 'make,iterator,value,done,tail,call') throw 'spread completes before outer await';

  let skippedOuter = 0;
  function unexpectedOuter() { skippedOuter++; throw 'optional outer argument'; }
  const nullFactory = {make() { return null; }};
  if ((nullFactory?.make(await 1))?.(await unexpectedOuter()) !== undefined || skippedOuter !== 0) throw 'optional outer Call skips a null completed Value';

  const targetPropertyTrace = [];
  const targetPropertySymbol = Symbol('target-property');
  const targetPropertyObject = {};
  let targetPropertyProxy;
  let targetPropertySourceReads = 0;
  let targetPropertyKeyReads = 0;
  let targetPropertyGets = 0;
  let targetPropertyCalls = 0;
  Object.defineProperty(targetPropertyObject, targetPropertySymbol, {configurable: true, get() {
    targetPropertyGets++;
    targetPropertyTrace.push('get');
    if (this !== targetPropertyProxy) throw 'target-await terminal Get receiver';
    return function(a, b, c, d, e) {
      'use strict';
      targetPropertyCalls++;
      targetPropertyTrace.push('call');
      if (this !== targetPropertyProxy || a !== 1 || b !== 2 || c !== 3 || d !== 4 || e !== 5) throw 'target-await retained Property Reference';
      return 115;
    };
  }});
  targetPropertyProxy = new Proxy(targetPropertyObject, {get(object, property, receiver) {
    if (property === 'then') return undefined;
    targetPropertyTrace.push('proxy-get');
    if (object !== targetPropertyObject || property !== targetPropertySymbol || receiver !== targetPropertyProxy) throw 'target-await Proxy Reference';
    return Reflect.get(object, property, receiver);
  }});
  let targetPropertySelected;
  const targetPropertyThenable = {get then() {
    targetPropertyTrace.push('target-then');
    targetPropertySelected = {};
    return resolve => { targetPropertyTrace.push('target-resolve'); resolve(targetPropertyProxy); };
  }};
  targetPropertySelected = targetPropertyThenable;
  function targetPropertySource() { targetPropertySourceReads++; targetPropertyTrace.push('target-source'); return targetPropertySelected; }
  const targetPropertyKey = {[Symbol.toPrimitive](hint) {
    targetPropertyKeyReads++;
    targetPropertyTrace.push('key');
    if (hint !== 'string') throw 'target-await key hint';
    return targetPropertySymbol;
  }};
  function targetPropertyFirst() {
    targetPropertyTrace.push('first');
    Object.defineProperty(targetPropertyObject, targetPropertySymbol, {configurable: true, value() { throw 'target-await callee reread'; }});
    return 1;
  }
  const targetPropertyIterable = {[Symbol.iterator]() {
    targetPropertyTrace.push('iterator');
    let step = 0;
    return {next() {
      const current = step++;
      targetPropertyTrace.push('next' + current);
      return {done: current === 2, value: current + 2};
    }};
  }};
  const targetPropertyOuter = {get then() {
    targetPropertyTrace.push('outer-then');
    return resolve => { targetPropertyTrace.push('outer-resolve'); resolve(4); };
  }};
  function targetPropertyLast() { targetPropertyTrace.push('last'); return 5; }
  if (((await targetPropertySource())?.[targetPropertyKey])(targetPropertyFirst(), ...targetPropertyIterable, await targetPropertyOuter, targetPropertyLast()) !== 115) throw 'target-await grouped Call result';
  if (targetPropertySourceReads !== 1 || targetPropertyKeyReads !== 1 || targetPropertyGets !== 1 || targetPropertyCalls !== 1 || targetPropertySelected === targetPropertyThenable) throw 'target-await observation counts';
  if (targetPropertyTrace.join(',') !== 'target-source,target-then,target-resolve,key,proxy-get,get,first,iterator,next0,next1,next2,outer-then,outer-resolve,last,call') throw 'target-await full capture and outer operand order';

  const targetPropertyPrimitiveSymbol = Symbol('target-property-primitive');
  let targetPropertyPrimitiveGets = 0;
  Object.defineProperty(String.prototype, targetPropertyPrimitiveSymbol, {configurable: true, get: function() {
    'use strict';
    targetPropertyPrimitiveGets++;
    if (this !== 'raw') throw 'target-await primitive Get receiver';
    return function(value) {
      'use strict';
      if (this !== 'raw' || value !== 7) throw 'target-await primitive Call receiver';
      return 117;
    };
  }});
  function targetPropertyPrimitiveArgument() {
    Object.defineProperty(String.prototype, targetPropertyPrimitiveSymbol, {configurable: true, value() { throw 'target-await primitive reread'; }});
    return 7;
  }
  if (((await 'raw')?.[targetPropertyPrimitiveSymbol])(await targetPropertyPrimitiveArgument()) !== 117 || targetPropertyPrimitiveGets !== 1) throw 'target-await raw primitive Reference';

  const targetPropertyChild = {method(value) { 'use strict'; if (this !== targetPropertyChild || value !== 8) throw 'target-await post-Call terminal receiver'; return 118; }};
  const targetPropertyRoot = {make() { 'use strict'; if (this !== targetPropertyRoot) throw 'target-await synchronous inner Call receiver'; return targetPropertyChild; }};
  if (((await targetPropertyRoot)?.make().method)(await 8) !== 118) throw 'target-await synchronous tail Property';
  let targetPropertyNestedGets = 0;
  const targetPropertyNested = {get method() {
    targetPropertyNestedGets++;
    return function(value) { 'use strict'; if (this !== targetPropertyNested || value !== 9) throw 'target-await first optional Call receiver'; return 119; };
  }};
  if (((await targetPropertyNested)?.method)?.(await 9) !== 119 || targetPropertyNestedGets !== 1) throw 'target-await recursively owned first Call';
  let targetPropertySkipped = 0;
  function targetPropertyForbidden() { targetPropertySkipped++; throw 'target-await shorted optional outer'; }
  if (((await null)?.method)?.(await targetPropertyForbidden()) !== undefined || targetPropertySkipped !== 0) throw 'target-await first optional Call skips outer operands';
}
run().then(() => print('grouped-optional-calls:ok'), error => print('unexpected:' + error));
trace.push('caller');
262;
