const trace = [];
const foreign = __lilaCreateRealm().global;
const foreignPrototype = foreign.TypeError.prototype;
const marker = new foreign.TypeError('marker');
foreign.TypeError = null;
const intrinsicPrototype = TypeError.prototype;
TypeError = undefined;
let assigned = 'kept';
let calls = 0;
let later = 0;
let closes = 0;
function forbidden() { later++; throw 'later argument'; }
function checkMarker(error) {
  if (error !== marker || Object.getPrototypeOf(error) !== foreignPrototype || assigned !== 'kept') throw 'foreign abrupt identity';
}
const target = {get method() {
  trace.push('get');
  return function() { calls++; throw 'unexpected call'; };
}};
const throwingGet = {get method() { trace.push('throwing-get'); throw marker; }};
const throwingCall = {method(value) {
  'use strict';
  if (this !== throwingCall || value !== 3) throw 'throwing Call Reference';
  trace.push('call');
  throw marker;
}};
const iterable = {[Symbol.iterator]() {
  trace.push('iterator');
  return {next() {
    trace.push('next');
    return {get done() { trace.push('done'); return false; }, get value() { trace.push('value'); throw marker; }};
  }, return() { closes++; throw 'argument spread must not close'; }};
}};
async function run() {
  try { assigned = target.method?.(await Promise.reject(marker), forbidden()); }
  catch (error) { checkMarker(error); trace.push('rejected'); }
  finally { await 0; trace.push('reject-finally'); }
  try { assigned = throwingGet.method?.(await forbidden()); }
  catch (error) { checkMarker(error); trace.push('get-thrown'); }
  finally { await 0; trace.push('get-finally'); }
  try { assigned = target.method?.(...iterable, await forbidden()); }
  catch (error) { checkMarker(error); trace.push('spread-thrown'); }
  finally { await 0; trace.push('spread-finally'); }
  try { assigned = throwingCall?.method(await 3); }
  catch (error) { checkMarker(error); trace.push('call-thrown'); }
  finally { await 0; trace.push('call-finally'); }
  const noncallable = {method: 0};
  try { assigned = noncallable.method?.(await 4); }
  catch (error) {
    if (Object.getPrototypeOf(error) !== intrinsicPrototype || assigned !== 'kept') throw 'execution Realm intrinsic';
    trace.push('noncallable-thrown');
  } finally { await 0; trace.push('noncallable-finally'); }
  if (calls !== 0 || later !== 0 || closes !== 0 || assigned !== 'kept') throw 'abrupt call operands';
  if (trace.join(',') !== 'get,rejected,reject-finally,throwing-get,get-thrown,get-finally,get,iterator,next,done,value,spread-thrown,spread-finally,call,call-thrown,call-finally,noncallable-thrown,noncallable-finally') throw 'abrupt and finally order';
}
run().then(() => print('optional-call-abrupt:ok'), error => print('unexpected:' + error));
262;
