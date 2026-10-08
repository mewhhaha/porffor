const trace = [];
const method = Symbol('method');
const target = {};
let selected;
let baseCalls = 0;
let gets = 0;
let keys = 0;
let calls = 0;
let proxy;
function original(a, b, c) {
  'use strict';
  trace.push('call');
  calls++;
  if (this !== proxy || a !== 1 || b !== 2 || c !== 3) throw 'saved Reference';
  return 31;
}
Object.defineProperty(target, method, {configurable: true, get() {
  gets++;
  trace.push('get');
  if (this !== proxy) throw 'getter receiver';
  return original;
}});
proxy = new Proxy(target, {get(object, key, receiver) {
  trace.push('proxy');
  if (object !== target || key !== method || receiver !== proxy) throw 'Proxy Reference';
  return Reflect.get(object, key, receiver);
}});
selected = proxy;
function base() { baseCalls++; trace.push('base'); return selected; }
const key = {[Symbol.toPrimitive](hint) {
  keys++;
  trace.push('key');
  if (hint !== 'string') throw 'key hint';
  return method;
}};
function keySource() { trace.push('key-source'); return key; }
function first() {
  trace.push('first');
  selected = {};
  Object.defineProperty(target, method, {configurable: true, value() { throw 'callee reread'; }});
  return 1;
}
const pending = {get then() {
  trace.push('arg-then');
  return resolve => { trace.push('arg-resolve'); resolve(2); };
}};
function last() { trace.push('last'); return 3; }
async function run() {
  const result = base()?.[await keySource()](first(), await pending, last());
  if (result !== 31 || baseCalls !== 1 || gets !== 1 || keys !== 1 || calls !== 1) throw 'repeated optional call observation';
  if (trace.join(',') !== 'base,key-source,caller,key,proxy,get,first,arg-then,arg-resolve,last,call') throw 'optional call order';

  const owner = {};
  let ownerGets = 0;
  Object.defineProperty(owner, 'method', {configurable: true, get() {
    ownerGets++;
    return function(value) {
      'use strict';
      if (this !== owner || value !== 22) throw 'first Call Reference';
      return 44;
    };
  }});
  function replaceOwnerMethod() {
    Object.defineProperty(owner, 'method', {configurable: true, value() { throw 'first Call reread'; }});
    return 22;
  }
  if (owner.method?.(await replaceOwnerMethod()) !== 44 || ownerGets !== 1) throw 'first Call capture';

  const primitiveMethod = Symbol('primitive');
  let primitiveGets = 0;
  Object.defineProperty(String.prototype, primitiveMethod, {configurable: true, get: function() {
    'use strict';
    primitiveGets++;
    if (this !== 'value') throw 'primitive getter receiver';
    return function(value) {
      'use strict';
      if (this !== 'value' || value !== 9) throw 'primitive call receiver';
      return 19;
    };
  }});
  function primitiveArgument() {
    Object.defineProperty(String.prototype, primitiveMethod, {configurable: true, value() { throw 'primitive reread'; }});
    return 9;
  }
  if ('value'[primitiveMethod]?.(await primitiveArgument()) !== 19 || primitiveGets !== 1) throw 'raw primitive Reference';

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
  function middle() { spreadTrace.push('middle'); return 6; }
  function tail() { spreadTrace.push('tail'); return 7; }
  if (spreadOwner.method?.(...iterable, await middle(), tail()) !== 47) throw 'spread result';
  if (spreadTrace.join(',') !== 'get,iterator,next0,next1,next2,middle,tail,call') throw 'spread evaluation order';

  const groupedTrace = [];
  const madeChild = {};
  const maker = {make(value) {
    'use strict';
    groupedTrace.push('make');
    if (this !== maker || value !== 10) throw 'grouped preceding Call';
    return madeChild;
  }};
  Object.defineProperty(madeChild, 'method', {configurable: true, get() {
    groupedTrace.push('get');
    return function(value) {
      'use strict';
      groupedTrace.push('call');
      if (this !== madeChild || value !== 11) throw 'grouped terminal Reference';
      return 21;
    };
  }});
  function groupedKey() { groupedTrace.push('key'); return 'method'; }
  function groupedArgument() {
    groupedTrace.push('outer');
    Object.defineProperty(madeChild, 'method', {configurable: true, value() { throw 'grouped callee reread'; }});
    return 11;
  }
  if ((maker?.make(await 10)[await groupedKey()])(await groupedArgument()) !== 21) throw 'grouped Call result';
  if (groupedTrace.join(',') !== 'make,key,get,outer,call') throw 'grouped preceding Call order';

  const nestedOwner = {};
  let nestedGets = 0;
  Object.defineProperty(nestedOwner, 'method', {configurable: true, get() {
    nestedGets++;
    return function(value) {
      'use strict';
      if (this !== nestedOwner || value !== 13) throw 'nested optional Call Reference';
      return 23;
    };
  }});
  function nestedArgument() {
    Object.defineProperty(nestedOwner, 'method', {configurable: true, value() { throw 'nested callee reread'; }});
    return 13;
  }
  if ((nestedOwner?.[await 'method'])?.(await nestedArgument()) !== 23 || nestedGets !== 1) throw 'first Call on grouped awaited Reference';

  let tagGets = 0;
  Object.defineProperty(madeChild, 'tag', {configurable: true, get() {
    tagGets++;
    return function(template, value) {
      'use strict';
      if (this !== madeChild || template[0] !== 'head' || template.raw[1] !== 'tail' || value !== 12) throw 'grouped tag Reference';
      return 22;
    };
  }});
  function tagArgument() {
    Object.defineProperty(madeChild, 'tag', {configurable: true, value() { throw 'grouped tag reread'; }});
    return 12;
  }
  if ((maker?.make(await 10).tag)`head${await tagArgument()}tail` !== 22 || tagGets !== 1) throw 'grouped tag after Call';
  if (eval?.(42, await Promise.resolve(0)) !== 42) throw 'optional eval non-string result';
}
run().then(() => print('optional-call-references:ok'), error => print('unexpected:' + error));
trace.push('caller');
262;
