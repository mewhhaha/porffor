const trace = [];
const foreign = __lilaCreateRealm().global;
const foreignPrototype = foreign.TypeError.prototype;
const marker = new foreign.TypeError('marker');
foreign.TypeError = null;
const intrinsicPrototype = TypeError.prototype;
TypeError = undefined;
let assigned = 'kept';
let skippedKeys = 0;
let badArguments = 0;
let calls = 0;
let closes = 0;
function skippedKey() { skippedKeys++; throw 'shorted key'; }
function badArgument() { badArguments++; throw 'argument must be skipped'; }
function checkMarker(error) {
  if (error !== marker || Object.getPrototypeOf(error) !== foreignPrototype || assigned !== 'kept') throw 'foreign abrupt identity or result';
}
function nullBase() { trace.push('null-base'); return null; }
function nullFirst() { trace.push('null-a'); return 1; }
const nullThenable = {get then() {
  trace.push('null-then');
  return resolve => { trace.push('null-resolve'); resolve(2); };
}};
function nullLast() { trace.push('null-c'); return 3; }
async function run() {
  try { assigned = (nullBase()?.[await skippedKey()])(nullFirst(), await nullThenable, nullLast()); }
  catch (error) {
    if (Object.getPrototypeOf(error) !== intrinsicPrototype || assigned !== 'kept') throw 'nullish grouped call TypeError';
    trace.push('null-thrown');
  } finally { await 0; trace.push('null-finally'); }
  const noncallable = {get method() { trace.push('noncallable-get'); return 0; }};
  function noncallableArgument() { trace.push('noncallable-arg'); return 4; }
  try { assigned = (noncallable?.[await 'method'])(await noncallableArgument()); }
  catch (error) {
    if (Object.getPrototypeOf(error) !== intrinsicPrototype || assigned !== 'kept') throw 'noncallable grouped call TypeError';
    trace.push('noncallable-thrown');
  } finally { await 0; trace.push('noncallable-finally'); }
  const target = {method() { calls++; throw 'unexpected call'; }};
  try { assigned = (target?.[await Promise.reject(marker)])(badArgument()); }
  catch (error) { checkMarker(error); trace.push('key-rejected'); }
  finally { await 0; trace.push('reject-finally'); }
  const throwingKey = {[Symbol.toPrimitive](hint) {
    if (hint !== 'string') throw 'key hint';
    trace.push('coerce');
    throw marker;
  }};
  try { assigned = (target?.[await throwingKey])(badArgument()); }
  catch (error) { checkMarker(error); trace.push('key-thrown'); }
  finally { await 0; trace.push('key-finally'); }
  const getter = {get method() { trace.push('getter'); throw marker; }};
  try { assigned = (getter?.[await 'method'])(badArgument()); }
  catch (error) { checkMarker(error); trace.push('get-thrown'); }
  finally { await 0; trace.push('get-finally'); }
  const proxy = new Proxy(target, {get() { trace.push('proxy'); throw marker; }});
  try { assigned = (proxy?.[await 'method'])(badArgument()); }
  catch (error) { checkMarker(error); trace.push('proxy-thrown'); }
  finally { await 0; trace.push('proxy-finally'); }
  const spreadTarget = {get method() { trace.push('spread-get'); return function() { calls++; throw 'spread callee'; }; }};
  const iterable = {[Symbol.iterator]() {
    trace.push('iterator');
    return {next() {
      trace.push('next');
      return {get done() { trace.push('done'); return false; }, get value() { trace.push('value'); throw marker; }};
    }, return() { closes++; throw 'argument-list abrupt must not close'; }};
  }};
  try { assigned = (spreadTarget?.[await 'method'])(...iterable, await badArgument(), badArgument()); }
  catch (error) { checkMarker(error); trace.push('spread-thrown'); }
  finally { await 0; trace.push('spread-finally'); }
  if (assigned !== 'kept' || skippedKeys !== 0 || badArguments !== 0 || calls !== 0 || closes !== 0) throw 'abrupt operand boundary';
  if (trace.join(',') !== 'null-base,null-a,null-then,caller,null-resolve,null-c,null-thrown,null-finally,noncallable-get,noncallable-arg,noncallable-thrown,noncallable-finally,key-rejected,reject-finally,coerce,key-thrown,key-finally,getter,get-thrown,get-finally,proxy,proxy-thrown,proxy-finally,spread-get,iterator,next,done,value,spread-thrown,spread-finally') throw 'abrupt and finally order';

  const callAbruptTrace = [];
  let valueErrors = 0;
  let skippedInner = 0;
  let skippedOuter = 0;
  let valueCloses = 0;
  function skippedInnerValue() { skippedInner++; throw 'shorted inner argument'; }
  function skippedOuterValue() { skippedOuter++; throw 'abrupt outer argument'; }
  function actualOuterValue() { callAbruptTrace.push('outer'); return 2; }
  function innerNullBase() { callAbruptTrace.push('null'); return null; }
  try { const result = (innerNullBase()?.make(await skippedInnerValue()))(await actualOuterValue()); assigned = result; }
  catch (error) {
    if (Object.getPrototypeOf(error) !== intrinsicPrototype || assigned !== 'kept') throw 'null returned callee intrinsic error';
    valueErrors++;
    callAbruptTrace.push('null-thrown');
  } finally { await 0; callAbruptTrace.push('null-finally'); }

  const valueNoncallable = {make() { callAbruptTrace.push('noncallable'); return 7; }};
  try { const result = (valueNoncallable?.make(await 1))(await actualOuterValue()); assigned = result; }
  catch (error) {
    if (Object.getPrototypeOf(error) !== intrinsicPrototype || assigned !== 'kept') throw 'returned noncallable error';
    valueErrors++;
    callAbruptTrace.push('noncallable-thrown');
  } finally { await 0; callAbruptTrace.push('noncallable-finally'); }

  const innerThrow = {make() { callAbruptTrace.push('inner-call'); throw marker; }};
  try { const result = (innerThrow?.make(await 1))(await skippedOuterValue()); assigned = result; }
  catch (error) { checkMarker(error); valueErrors++; callAbruptTrace.push('inner-thrown'); }
  finally { await 0; callAbruptTrace.push('inner-finally'); }
  try { const result = (innerThrow?.make(await Promise.reject(marker)))(await skippedOuterValue()); assigned = result; }
  catch (error) { checkMarker(error); valueErrors++; callAbruptTrace.push('inner-rejected'); }
  finally { await 0; callAbruptTrace.push('reject-finally'); }

  const outerThrow = {make() {
    callAbruptTrace.push('factory');
    return function(value) {
      'use strict';
      if (this !== undefined || value !== 2) throw 'throwing returned Call Value';
      callAbruptTrace.push('returned-call');
      throw marker;
    };
  }};
  try { const result = (outerThrow?.make(await 1))(await actualOuterValue()); assigned = result; }
  catch (error) { checkMarker(error); valueErrors++; callAbruptTrace.push('outer-thrown'); }
  finally { await 0; callAbruptTrace.push('outer-finally'); }

  const valueIterable = {[Symbol.iterator]() {
    callAbruptTrace.push('iterator');
    return {next() {
      callAbruptTrace.push('next');
      return {done: false, get value() { callAbruptTrace.push('value'); throw marker; }};
    }, return() { valueCloses++; throw 'argument value close'; }};
  }};
  try { const result = (outerThrow?.make(await 1))(...valueIterable, await skippedOuterValue()); assigned = result; }
  catch (error) { checkMarker(error); valueErrors++; callAbruptTrace.push('spread-thrown'); }
  finally { await 0; callAbruptTrace.push('spread-finally'); }

  const missingNested = null;
  try { const result = ((missingNested?.[await skippedInnerValue()])?.())(await actualOuterValue()); assigned = result; }
  catch (error) {
    if (Object.getPrototypeOf(error) !== intrinsicPrototype || assigned !== 'kept') throw 'nested returned null callee error';
    valueErrors++;
    callAbruptTrace.push('nested-thrown');
  } finally { await 0; callAbruptTrace.push('nested-finally'); }
  if (valueErrors !== 7 || skippedInner !== 0 || skippedOuter !== 0 || valueCloses !== 0 || assigned !== 'kept') throw 'grouped Call value abrupt boundaries';
  if (callAbruptTrace.join(',') !== 'null,outer,null-thrown,null-finally,noncallable,outer,noncallable-thrown,noncallable-finally,inner-call,inner-thrown,inner-finally,inner-rejected,reject-finally,factory,outer,returned-call,outer-thrown,outer-finally,factory,iterator,next,value,spread-thrown,spread-finally,outer,nested-thrown,nested-finally') throw 'grouped Call value abrupt order';

  const targetAbruptTrace = [];
  let targetErrors = 0;
  let targetSkippedKeys = 0;
  let targetSkippedArguments = 0;
  let targetUnexpectedCalls = 0;
  let targetCloses = 0;
  function targetForbiddenKey() { targetSkippedKeys++; throw 'target-only skipped key'; }
  function targetForbiddenArgument() { targetSkippedArguments++; throw 'target-only skipped argument'; }
  function targetOuterArgument() { targetAbruptTrace.push('outer'); return 2; }
  const targetNullThenable = {then(resolve) {
    targetAbruptTrace.push('null-then');
    resolve(null);
  }};
  try { assigned = ((await targetNullThenable)?.[targetForbiddenKey()])(targetOuterArgument(), await 3, targetOuterArgument()); }
  catch (error) {
    if (Object.getPrototypeOf(error) !== intrinsicPrototype || assigned !== 'kept') throw 'target-only null intrinsic error';
    targetErrors++;
    targetAbruptTrace.push('null-thrown');
  } finally { await 0; targetAbruptTrace.push('null-finally'); }

  const targetNoncallable = {get method() { targetAbruptTrace.push('noncallable-get'); return 0; }};
  try { assigned = ((await targetNoncallable)?.method)(await targetOuterArgument()); }
  catch (error) {
    if (Object.getPrototypeOf(error) !== intrinsicPrototype || assigned !== 'kept') throw 'target-only noncallable intrinsic error';
    targetErrors++;
    targetAbruptTrace.push('noncallable-thrown');
  } finally { await 0; targetAbruptTrace.push('noncallable-finally'); }

  try { assigned = ((await Promise.reject(marker))?.[targetForbiddenKey()])(await targetForbiddenArgument()); }
  catch (error) { checkMarker(error); targetErrors++; targetAbruptTrace.push('target-rejected'); }
  finally { await 0; targetAbruptTrace.push('target-finally'); }

  const targetKey = {[Symbol.toPrimitive](hint) {
    if (hint !== 'string') throw 'target-only key hint';
    targetAbruptTrace.push('coerce');
    throw marker;
  }};
  const targetUnused = {method() { targetUnexpectedCalls++; throw 'target-only unexpected call'; }};
  try { assigned = ((await targetUnused)?.[targetKey])(await targetForbiddenArgument()); }
  catch (error) { checkMarker(error); targetErrors++; targetAbruptTrace.push('key-thrown'); }
  finally { await 0; targetAbruptTrace.push('key-finally'); }

  const targetGetter = {get method() { targetAbruptTrace.push('getter'); throw marker; }};
  try { assigned = ((await targetGetter)?.method)(await targetForbiddenArgument()); }
  catch (error) { checkMarker(error); targetErrors++; targetAbruptTrace.push('get-thrown'); }
  finally { await 0; targetAbruptTrace.push('get-finally'); }

  const targetProxy = new Proxy(targetUnused, {get(owner, key) {
    if (key === 'then') return undefined;
    targetAbruptTrace.push('proxy');
    throw marker;
  }});
  try { assigned = ((await targetProxy)?.method)(await targetForbiddenArgument()); }
  catch (error) { checkMarker(error); targetErrors++; targetAbruptTrace.push('proxy-thrown'); }
  finally { await 0; targetAbruptTrace.push('proxy-finally'); }

  const targetRetained = {get method() {
    targetAbruptTrace.push('retained-get');
    return function() { targetUnexpectedCalls++; throw 'rejected outer await must skip call'; };
  }};
  try { assigned = ((await targetRetained)?.method)(await Promise.reject(marker), targetForbiddenArgument()); }
  catch (error) { checkMarker(error); targetErrors++; targetAbruptTrace.push('outer-rejected'); }
  finally { await 0; targetAbruptTrace.push('outer-finally'); }

  const targetThrowing = {get method() {
    targetAbruptTrace.push('throwing-get');
    return function(value) {
      'use strict';
      if (this !== targetThrowing || value !== 2) throw 'target-only throwing receiver';
      targetAbruptTrace.push('call');
      throw marker;
    };
  }};
  try { assigned = ((await targetThrowing)?.method)(await targetOuterArgument()); }
  catch (error) { checkMarker(error); targetErrors++; targetAbruptTrace.push('call-thrown'); }
  finally { await 0; targetAbruptTrace.push('call-finally'); }

  const targetSpread = {get method() {
    targetAbruptTrace.push('spread-get');
    return function() { targetUnexpectedCalls++; throw 'target-only spread must skip call'; };
  }};
  const targetIterable = {[Symbol.iterator]() {
    targetAbruptTrace.push('iterator');
    return {next() {
      targetAbruptTrace.push('next');
      return {get done() { targetAbruptTrace.push('done'); return false; }, get value() { targetAbruptTrace.push('value'); throw marker; }};
    }, return() { targetCloses++; throw 'target-only argument-list abrupt must not close'; }};
  }};
  try { assigned = ((await targetSpread)?.method)(...targetIterable, await targetForbiddenArgument()); }
  catch (error) { checkMarker(error); targetErrors++; targetAbruptTrace.push('spread-thrown'); }
  finally { await 0; targetAbruptTrace.push('spread-finally'); }
  if (targetErrors !== 9 || targetSkippedKeys !== 0 || targetSkippedArguments !== 0 || targetUnexpectedCalls !== 0 || targetCloses !== 0 || assigned !== 'kept') throw 'target-only abrupt boundaries';
  if (targetAbruptTrace.join(',') !== 'null-then,outer,outer,null-thrown,null-finally,noncallable-get,outer,noncallable-thrown,noncallable-finally,target-rejected,target-finally,coerce,key-thrown,key-finally,getter,get-thrown,get-finally,proxy,proxy-thrown,proxy-finally,retained-get,outer-rejected,outer-finally,throwing-get,outer,call,call-thrown,call-finally,spread-get,iterator,next,done,value,spread-thrown,spread-finally') throw 'target-only abrupt order';

}
run().then(() => print('grouped-optional-abrupt:ok'), error => print('unexpected:' + error));
trace.push('caller');
262;
