const trace = [];
const symbol = Symbol('tag');
const target = {};
const replacement = {[symbol]() { throw 'replacement tag'; }};
const intrinsicPrototype = TypeError.prototype;
const foreign = __lilaCreateRealm().global;
const foreignPrototype = foreign.TypeError.prototype;
const marker = new foreign.TypeError('tag marker');
foreign.TypeError = null;
TypeError = undefined;
let proxy;
let current;
let template;
let tagCalls = 0;
let baseCalls = 0;
let keyConversions = 0;
let proxyGets = 0;
let getterCalls = 0;
let skippedKeys = 0;
const substitution = {[Symbol.toPrimitive]() { throw 'tagged substitution coerced'; }};
const secondValue = Symbol('substitution');
function originalTag(strings, first, second) {
  'use strict';
  tagCalls++;
  trace.push('tag' + tagCalls);
  if (this !== proxy || first !== substitution || second !== secondValue) throw 'tag Reference or substitution identity';
  if (!Object.isFrozen(strings) || !Object.isFrozen(strings.raw) || strings.length !== 3 || strings.raw.length !== 3) throw 'template frozen arrays';
  if (strings[0] !== 'head\n' || strings.raw[0] !== 'head\\n' || strings[1] !== 'mid' || strings[2] !== 'tail') throw 'cooked/raw template';
  const rawDescriptor = Object.getOwnPropertyDescriptor(strings, 'raw');
  const firstDescriptor = Object.getOwnPropertyDescriptor(strings, '0');
  if (rawDescriptor.writable || rawDescriptor.enumerable || rawDescriptor.configurable || firstDescriptor.writable || firstDescriptor.configurable) throw 'template descriptors';
  if (template === undefined) template = strings;
  else if (template !== strings || template.raw !== strings.raw) throw 'template site cache';
  return tagCalls;
}
function install() {
  Object.defineProperty(target, symbol, {configurable: true, get() {
    if (this !== proxy) throw 'tag accessor receiver';
    getterCalls++;
    trace.push('tag-get');
    return originalTag;
  }});
}
proxy = new Proxy(target, {get(object, key, receiver) {
  if (object !== target || key !== symbol || receiver !== proxy) throw 'tag Proxy Reference';
  proxyGets++;
  trace.push('proxy-get');
  return Reflect.get(object, key, receiver);
}});
current = proxy;
install();
function base() { baseCalls++; trace.push('base'); return current; }
const key = {[Symbol.toPrimitive](hint) {
  if (hint !== 'string') throw 'tag key hint';
  keyConversions++;
  trace.push('key');
  return symbol;
}};
function keySource() { trace.push('key-source'); return key; }
function firstSubstitution() {
  trace.push('sub-source');
  return {get then() {
    trace.push('sub-then');
    current = replacement;
    Object.defineProperty(target, symbol, {configurable: true, value() { throw 'reread tag'; }});
    return resolve => { trace.push('sub-resolve'); resolve(substitution); };
  }};
}
function secondSubstitution() { trace.push('sub-second'); current = replacement; return secondValue; }
async function render() {
  return (base()?.[await keySource()])`head\n${await firstSubstitution()}mid${secondSubstitution()}tail`;
}
function skippedKey() { skippedKeys++; throw 'nullish tag key'; }
function nullTagBase() { trace.push('null-base'); return undefined; }
function nullSubstitution() { trace.push('null-sub'); return 3; }
function nullLast() { trace.push('null-last'); return 4; }
async function run() {
  if (await render() !== 1) throw 'first tag result';
  current = proxy;
  install();
  if (await render() !== 2) throw 'second tag result';
  if (baseCalls !== 2 || keyConversions !== 2 || proxyGets !== 2 || getterCalls !== 2 || tagCalls !== 2 || current !== replacement) throw 'tag hook count';
  if (trace.join(',') !== 'base,key-source,caller,key,proxy-get,tag-get,sub-source,sub-then,sub-resolve,sub-second,tag1,base,key-source,key,proxy-get,tag-get,sub-source,sub-then,sub-resolve,sub-second,tag2') throw 'tag capture and substitution order';
  const completedTraceLength = trace.length;
  let assigned = 'kept';
  try { assigned = (nullTagBase()?.[await skippedKey()])`x${await nullSubstitution()}y${nullLast()}z`; }
  catch (error) {
    if (Object.getPrototypeOf(error) !== intrinsicPrototype || assigned !== 'kept') throw 'grouped nullish tag TypeError';
    trace.push('null-thrown');
  } finally { await 0; trace.push('null-finally'); }
  const throwingTag = {get tag() {
    trace.push('foreign-get');
    return function(strings, a, b) {
      'use strict';
      if (this !== throwingTag || a !== 41 || b !== 42 || !Object.isFrozen(strings) || !Object.isFrozen(strings.raw)) throw 'throwing tag arguments';
      trace.push('foreign-tag');
      throw marker;
    };
  }};
  function foreignFirst() { trace.push('foreign-first'); return 41; }
  function foreignLast() { trace.push('foreign-last'); return 42; }
  try { assigned = (throwingTag?.[await 'tag'])`a${await foreignFirst()}b${foreignLast()}c`; }
  catch (error) {
    if (error !== marker || Object.getPrototypeOf(error) !== foreignPrototype || assigned !== 'kept') throw 'foreign tag abrupt identity';
    trace.push('foreign-thrown');
  } finally { await 0; trace.push('foreign-finally'); }
  const primitiveSymbol = Symbol('primitive-tag');
  Object.defineProperty(String.prototype, primitiveSymbol, {configurable: true, get: function() {
    'use strict';
    if (this !== 'primitive') throw 'primitive tag getter receiver';
    trace.push('primitive-get');
    return function(strings, value) {
      'use strict';
      trace.push('primitive-tag');
      if (this !== 'primitive' || value !== 7 || strings[0] !== 'p' || strings[1] !== 'q' || !Object.isFrozen(strings)) throw 'primitive tag Reference';
      return 17;
    };
  }});
  function primitiveSubstitution() {
    trace.push('primitive-sub');
    Object.defineProperty(String.prototype, primitiveSymbol, {configurable: true, value() { throw 'primitive tag reread'; }});
    return 7;
  }
  if (('primitive'?.[await primitiveSymbol])`p${await primitiveSubstitution()}q` !== 17 || skippedKeys !== 0) throw 'primitive or nullish tag';
  if (trace.slice(completedTraceLength).join(',') !== 'null-base,null-sub,null-last,null-thrown,null-finally,foreign-get,foreign-first,foreign-last,foreign-tag,foreign-thrown,foreign-finally,primitive-get,primitive-sub,primitive-tag') throw 'tag abrupt/substitution order';

  const callTagTrace = [];
  let callTagTemplate;
  let callTagGets = 0;
  let callTagCalls = 0;
  const callTagFactory = {};
  function returnedTag(strings, value) {
    'use strict';
    callTagCalls++;
    callTagTrace.push('tag');
    if (this !== undefined || value !== substitution || strings[0] !== 'left' || strings[1] !== 'right') throw 'returned tag Value or substitution';
    if (!Object.isFrozen(strings) || !Object.isFrozen(strings.raw)) throw 'returned tag frozen template';
    if (callTagTemplate === undefined) callTagTemplate = strings;
    else if (strings !== callTagTemplate || strings.raw !== callTagTemplate.raw) throw 'returned tag site cache';
    return callTagCalls;
  }
  function installCallTag() {
    Object.defineProperty(callTagFactory, 'make', {configurable: true, get() {
      callTagGets++;
      callTagTrace.push('get');
      return function(value) {
        'use strict';
        callTagTrace.push('make');
        if (this !== callTagFactory || value !== 1) throw 'inner tag factory Reference';
        return returnedTag;
      };
    }});
  }
  function callTagArgument() { callTagTrace.push('argument'); return 1; }
  function callTagSubstitution() {
    callTagTrace.push('substitution');
    Object.defineProperty(callTagFactory, 'make', {configurable: true, value() { throw 'tag factory reread'; }});
    return substitution;
  }
  async function renderCallTag() {
    return (callTagFactory?.make(await callTagArgument()))`left${await callTagSubstitution()}right`;
  }
  installCallTag();
  if (await renderCallTag() !== 1) throw 'first returned tag';
  installCallTag();
  if (await renderCallTag() !== 2 || callTagGets !== 2 || callTagCalls !== 2) throw 'returned tag observation count';
  if (callTagTrace.join(',') !== 'get,argument,make,substitution,tag,get,argument,make,substitution,tag') throw 'returned tag capture precedes substitutions';

  const nestedTagFactory = {make() {
    'use strict';
    if (this !== nestedTagFactory) throw 'target-only tag inner Reference';
    return function(strings, value) {
      'use strict';
      if (this !== undefined || value !== 19 || strings[0] !== 'n' || strings[1] !== 't' || !Object.isFrozen(strings)) throw 'target-only returned tag Value';
      return 89;
    };
  }};
  if (((nestedTagFactory?.[await 'make'])?.())`n${await 19}t` !== 89) throw 'nested target-only tag result';

  const tagAbruptTrace = [];
  let tagValueErrors = 0;
  let skippedTagInner = 0;
  let skippedTagOuter = 0;
  function skippedTagArgument() { skippedTagInner++; throw 'shorted tag factory argument'; }
  function skippedTagSubstitution() { skippedTagOuter++; throw 'abrupt tag substitution'; }
  function actualTagSubstitution() { tagAbruptTrace.push('substitution'); return substitution; }
  const nullTagFactory = null;
  try { const result = (nullTagFactory?.make(await skippedTagArgument()))`x${await actualTagSubstitution()}y`; assigned = result; }
  catch (error) {
    if (Object.getPrototypeOf(error) !== intrinsicPrototype || assigned !== 'kept') throw 'null returned tag intrinsic error';
    tagValueErrors++;
    tagAbruptTrace.push('null-thrown');
  }
  const badTagFactory = {make() { tagAbruptTrace.push('bad-factory'); return 7; }};
  try { const result = (badTagFactory?.make(await 1))`x${await actualTagSubstitution()}y`; assigned = result; }
  catch (error) {
    if (Object.getPrototypeOf(error) !== intrinsicPrototype || assigned !== 'kept') throw 'noncallable returned tag intrinsic error';
    tagValueErrors++;
    tagAbruptTrace.push('bad-thrown');
  }
  const throwingTagFactory = {make() { tagAbruptTrace.push('throwing-factory'); throw marker; }};
  try { const result = (throwingTagFactory?.make(await 1))`x${await skippedTagSubstitution()}y`; assigned = result; }
  catch (error) {
    if (error !== marker || Object.getPrototypeOf(error) !== foreignPrototype || assigned !== 'kept') throw 'tag factory abrupt identity';
    tagValueErrors++;
    tagAbruptTrace.push('factory-thrown');
  }
  const returnedThrowingTag = {make() {
    tagAbruptTrace.push('factory');
    return function(strings, value) {
      'use strict';
      if (this !== undefined || value !== substitution || !Object.isFrozen(strings) || !Object.isFrozen(strings.raw)) throw 'throwing tag Value arguments';
      tagAbruptTrace.push('tag');
      throw marker;
    };
  }};
  try { const result = (returnedThrowingTag?.make(await 1))`x${await actualTagSubstitution()}y`; assigned = result; }
  catch (error) {
    if (error !== marker || Object.getPrototypeOf(error) !== foreignPrototype || assigned !== 'kept') throw 'returned tag abrupt identity';
    tagValueErrors++;
    tagAbruptTrace.push('tag-thrown');
  }
  if (tagValueErrors !== 4 || skippedTagInner !== 0 || skippedTagOuter !== 0 || assigned !== 'kept') throw 'returned tag abrupt boundaries';
  if (tagAbruptTrace.join(',') !== 'substitution,null-thrown,bad-factory,substitution,bad-thrown,throwing-factory,factory-thrown,factory,substitution,tag,tag-thrown') throw 'returned tag substitution and abrupt order';

  const targetPropertyTagTrace = [];
  let targetPropertyTagTemplate;
  let targetPropertyTagGets = 0;
  let targetPropertyTagCalls = 0;
  const targetPropertyTagOwner = {};
  function targetPropertyTagGetter() {
    targetPropertyTagGets++;
    targetPropertyTagTrace.push('get');
    if (this !== targetPropertyTagOwner) throw 'target-await tag Get receiver';
    return function(strings, value) {
      'use strict';
      targetPropertyTagCalls++;
      targetPropertyTagTrace.push('tag');
      if (this !== targetPropertyTagOwner || value !== substitution) throw 'target-await tag Reference';
      if (!Object.isFrozen(strings) || !Object.isFrozen(strings.raw) || strings.length !== 2 || strings[0] !== 'left\n' || strings.raw[0] !== 'left\\n' || strings[1] !== 'right') throw 'target-await original frozen template';
      if (targetPropertyTagTemplate === undefined) targetPropertyTagTemplate = strings;
      else if (strings !== targetPropertyTagTemplate || strings.raw !== targetPropertyTagTemplate.raw) throw 'target-await repeated template site';
      return targetPropertyTagCalls;
    };
  }
  function targetPropertyTagInstall() { Object.defineProperty(targetPropertyTagOwner, 'tag', {configurable: true, get: targetPropertyTagGetter}); }
  const targetPropertyTagSubstitution = {get then() {
    targetPropertyTagTrace.push('sub-then');
    Object.defineProperty(targetPropertyTagOwner, 'tag', {configurable: true, value() { throw 'target-await tag reread'; }});
    return resolve => { targetPropertyTagTrace.push('sub-resolve'); resolve(substitution); };
  }};
  async function targetPropertyRender() { return ((await targetPropertyTagOwner)?.tag)`left\n${await targetPropertyTagSubstitution}right`; }
  targetPropertyTagInstall();
  if (await targetPropertyRender() !== 1) throw 'target-await first tag result';
  targetPropertyTagInstall();
  if (await targetPropertyRender() !== 2 || targetPropertyTagGets !== 2 || targetPropertyTagCalls !== 2) throw 'target-await repeated tag count';
  if (targetPropertyTagTrace.join(',') !== 'get,sub-then,sub-resolve,tag,get,sub-then,sub-resolve,tag') throw 'target-await tag capture before substitutions';

  const targetPropertyNullTagTrace = [];
  let targetPropertyTagSkipped = 0;
  function targetPropertyTagKey() { targetPropertyTagSkipped++; throw 'target-await null tag key'; }
  function targetPropertyTagFirst() { targetPropertyNullTagTrace.push('first'); return 4; }
  function targetPropertyTagLast() { targetPropertyNullTagTrace.push('last'); return 5; }
  try { assigned = ((await null)?.[targetPropertyTagKey()])`x${await targetPropertyTagFirst()}y${targetPropertyTagLast()}z`; }
  catch (error) {
    if (Object.getPrototypeOf(error) !== intrinsicPrototype || assigned !== 'kept') throw 'target-await null tag intrinsic completion';
    targetPropertyNullTagTrace.push('throw');
  } finally { await 0; targetPropertyNullTagTrace.push('finally'); }
  if (targetPropertyTagSkipped !== 0 || targetPropertyNullTagTrace.join(',') !== 'first,last,throw,finally') throw 'target-await null tag ordinary substitution order';
  const targetPropertyThrowingTag = {get tag() {
    targetPropertyNullTagTrace.push('get');
    return function(strings, value) {
      'use strict';
      if (this !== targetPropertyThrowingTag || value !== substitution || !Object.isFrozen(strings) || !Object.isFrozen(strings.raw)) throw 'target-await throwing tag Reference';
      targetPropertyNullTagTrace.push('tag');
      throw marker;
    };
  }};
  try { assigned = ((await targetPropertyThrowingTag)?.tag)`x${await substitution}y`; }
  catch (error) {
    if (error !== marker || Object.getPrototypeOf(error) !== foreignPrototype || assigned !== 'kept') throw 'target-await foreign tag abrupt identity';
    targetPropertyNullTagTrace.push('foreign-throw');
  } finally { await 0; targetPropertyNullTagTrace.push('foreign-finally'); }
  if (targetPropertyNullTagTrace.join(',') !== 'first,last,throw,finally,get,tag,foreign-throw,foreign-finally') throw 'target-await tag original abrupt and finally order';
}
run().then(() => print('grouped-optional-templates:ok'), error => print('unexpected:' + error));
trace.push('caller');
262;
