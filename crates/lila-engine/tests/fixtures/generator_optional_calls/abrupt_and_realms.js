function check(result, value, done) {
  if (!Object.is(result.value, value) || result.done !== done) throw 'iterator result';
}
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
function forbidden() { later++; throw 'later operand'; }
function checkMarker(error) {
  if (error !== marker || Object.getPrototypeOf(error) !== foreignPrototype || assigned !== 'kept') throw 'foreign abrupt identity';
}
const target = {get method() {
  trace.push('get');
  return function() { calls++; throw 'unexpected Call'; };
}};
function* pending() {
  try { const result = target.method?.(yield 'argument', forbidden()); assigned = result; }
  finally { trace.push('finally'); }
}
const thrown = pending();
check(thrown.next(), 'argument', false);
let caught;
try { thrown.throw(marker); } catch (error) { caught = error; }
checkMarker(caught);
check(thrown.next(), undefined, true);
const returned = pending();
check(returned.next(), 'argument', false);
check(returned.return(77), 77, true);
check(returned.next(), undefined, true);

const throwingGet = {get method() { trace.push('throwing-get'); throw marker; }};
function* getterFailure() {
  try { const result = throwingGet.method?.(yield forbidden()); assigned = result; }
  finally { trace.push('get-finally'); }
}
caught = undefined;
try { getterFailure().next(); } catch (error) { caught = error; }
checkMarker(caught);

const iterable = {[Symbol.iterator]() {
  trace.push('iterator');
  return {next() {
    trace.push('next');
    return {get done() { trace.push('done'); return false; }, get value() { trace.push('value'); throw marker; }};
  }, return() { closes++; throw 'argument spread must not close'; }};
}};
function* spreadFailure() {
  try { const result = target.method?.(...iterable, yield forbidden()); assigned = result; }
  finally { trace.push('spread-finally'); }
}
caught = undefined;
try { spreadFailure().next(); } catch (error) { caught = error; }
checkMarker(caught);

const throwingCall = {method(value, extra) {
  'use strict';
  trace.push('call');
  if (this !== throwingCall || value !== 3 || extra !== 4) throw 'throwing Call Reference';
  throw marker;
}};
function extra() { trace.push('extra'); return 4; }
function* callFailure() {
  try { const result = throwingCall?.method(yield 'call', extra()); assigned = result; }
  finally { trace.push('call-finally'); }
}
const callIterator = callFailure();
check(callIterator.next(), 'call', false);
caught = undefined;
try { callIterator.next(3); } catch (error) { caught = error; }
checkMarker(caught);

let childGets = 0;
let creates = 0;
const child = {get method() { childGets++; throw 'unentered later key'; }};
const factory = {create() { creates++; return child; }};
function* lateReturn() {
  try { return factory?.create(yield 'first')?.[yield 'second'](yield 'third'); }
  finally { trace.push('late-finally'); }
}
const late = lateReturn();
check(late.next(), 'first', false);
check(late.next(1), 'second', false);
check(late.return(99), 99, true);
if (creates !== 1 || childGets !== 0) throw 'Return must bypass later property and Call';

const noncallable = {method: 0};
function* intrinsicFailure() {
  try { const result = noncallable.method?.(yield 'noncallable'); assigned = result; }
  finally { trace.push('noncallable-finally'); }
}
const invalid = intrinsicFailure();
check(invalid.next(), 'noncallable', false);
caught = undefined;
try { invalid.next(4); } catch (error) { caught = error; }
if (Object.getPrototypeOf(caught) !== intrinsicPrototype || assigned !== 'kept') throw 'execution Realm intrinsic';
if (calls !== 0 || later !== 0 || closes !== 0) throw 'abrupt suffix effects';
if (trace.join(',') !== 'get,finally,get,finally,throwing-get,get-finally,get,iterator,next,done,value,spread-finally,extra,call,call-finally,late-finally,noncallable-finally') throw 'abrupt and finally order';

const groupedAbruptTrace = [];
let groupedOuterEffects = 0;
let groupedOuterCalls = 0;
let groupedCloses = 0;
function groupedLater() { groupedOuterEffects++; throw 'grouped later operand'; }
const groupedAbruptOwner = {make(value) {
  'use strict';
  groupedAbruptTrace.push('factory');
  if (this !== groupedAbruptOwner || value !== 1) throw 'grouped abrupt inner Reference';
  return function(value) {
    'use strict';
    groupedOuterCalls++;
    if (this !== undefined || value !== 2) throw 'grouped abrupt outer Value';
    throw marker;
  };
}};
function* groupedPendingInner() {
  try { const result = (groupedAbruptOwner?.make(yield 'grouped-inner'))(groupedLater()); assigned = result; }
  finally { groupedAbruptTrace.push('inner-finally'); }
}
const groupedInjectedInner = groupedPendingInner();
check(groupedInjectedInner.next(), 'grouped-inner', false);
caught = undefined;
try { groupedInjectedInner.throw(marker); } catch (error) { caught = error; }
checkMarker(caught);
const groupedReturnedInner = groupedPendingInner();
check(groupedReturnedInner.next(), 'grouped-inner', false);
check(groupedReturnedInner.return(88), 88, true);

function* groupedPendingOuter() {
  try { const result = (groupedAbruptOwner?.make(yield 'grouped-factory'))(yield 'grouped-outer'); assigned = result; }
  finally { groupedAbruptTrace.push('outer-finally'); }
}
const groupedInjectedOuter = groupedPendingOuter();
check(groupedInjectedOuter.next(), 'grouped-factory', false);
check(groupedInjectedOuter.next(1), 'grouped-outer', false);
caught = undefined;
try { groupedInjectedOuter.throw(marker); } catch (error) { caught = error; }
checkMarker(caught);
const groupedReturnedOuter = groupedPendingOuter();
check(groupedReturnedOuter.next(), 'grouped-factory', false);
check(groupedReturnedOuter.next(1), 'grouped-outer', false);
check(groupedReturnedOuter.return(89), 89, true);
const groupedThrownOuter = groupedPendingOuter();
check(groupedThrownOuter.next(), 'grouped-factory', false);
check(groupedThrownOuter.next(1), 'grouped-outer', false);
caught = undefined;
try { groupedThrownOuter.next(2); } catch (error) { caught = error; }
checkMarker(caught);

const groupedThrowingFactory = {make() { groupedAbruptTrace.push('factory-throw'); throw marker; }};
function* groupedFactoryFailure() {
  try { const result = (groupedThrowingFactory?.make(yield 'grouped-before-factory'))(groupedLater()); assigned = result; }
  finally { groupedAbruptTrace.push('factory-finally'); }
}
const groupedFactoryIterator = groupedFactoryFailure();
check(groupedFactoryIterator.next(), 'grouped-before-factory', false);
caught = undefined;
try { groupedFactoryIterator.next(1); } catch (error) { caught = error; }
checkMarker(caught);

const groupedThrowingIterable = {[Symbol.iterator]() {
  groupedAbruptTrace.push('iterator');
  return {next() {
    return {done: false, get value() { groupedAbruptTrace.push('value'); throw marker; }};
  }, return() { groupedCloses++; throw 'grouped argument IteratorClose'; }};
}};
function* groupedSpreadFailure() {
  try { const result = (groupedAbruptOwner?.make(yield 'grouped-spread-factory'))(...groupedThrowingIterable, yield groupedLater()); assigned = result; }
  finally { groupedAbruptTrace.push('spread-finally'); }
}
const groupedSpreadFailureIterator = groupedSpreadFailure();
check(groupedSpreadFailureIterator.next(), 'grouped-spread-factory', false);
caught = undefined;
try { groupedSpreadFailureIterator.next(1); } catch (error) { caught = error; }
checkMarker(caught);

const groupedTagFailureOwner = {make() { return function() { 'use strict'; if (this !== undefined) throw 'grouped tag receiver'; throw marker; }; }};
function* groupedTagFailure() {
  try { const result = (groupedTagFailureOwner?.make(yield 'grouped-tag-factory'))`head${yield 'grouped-tag-outer'}tail`; assigned = result; }
  finally { groupedAbruptTrace.push('tag-finally'); }
}
const groupedTagFailureIterator = groupedTagFailure();
check(groupedTagFailureIterator.next(), 'grouped-tag-factory', false);
check(groupedTagFailureIterator.next(1), 'grouped-tag-outer', false);
caught = undefined;
try { groupedTagFailureIterator.next(2); } catch (error) { caught = error; }
checkMarker(caught);
if (groupedOuterEffects !== 0 || groupedOuterCalls !== 1 || groupedCloses !== 0) throw 'grouped abrupt suffix effects';
if (groupedAbruptTrace.join(',') !== 'inner-finally,inner-finally,factory,outer-finally,factory,outer-finally,factory,outer-finally,factory-throw,factory-finally,factory,iterator,value,spread-finally,tag-finally') throw 'grouped abrupt finally order';
const groupedPropertyAbruptTrace = [];
let groupedPropertyGets = 0;
let groupedPropertyCalls = 0;
let groupedPropertyLaterEffects = 0;
let groupedPropertyCloses = 0;
const groupedPropertyAbruptOwner = {get method() {
  groupedPropertyGets++;
  groupedPropertyAbruptTrace.push('get');
  return function(value, extra) {
    'use strict';
    groupedPropertyCalls++;
    groupedPropertyAbruptTrace.push('call');
    if (this !== groupedPropertyAbruptOwner || value !== 2 || extra !== 3) throw 'grouped Property abrupt Call Reference';
    throw marker;
  };
}};
function groupedPropertyLast() { groupedPropertyLaterEffects++; groupedPropertyAbruptTrace.push('last'); return 3; }
function* groupedPropertyPending() {
  try { const result = (groupedPropertyAbruptOwner?.[yield 'grouped-property-key'])(yield 'grouped-property-outer', groupedPropertyLast()); assigned = result; }
  finally { groupedPropertyAbruptTrace.push('finally'); }
}
const groupedPropertyBeforeThrow = groupedPropertyPending();
check(groupedPropertyBeforeThrow.next(), 'grouped-property-key', false);
caught = undefined;
try { groupedPropertyBeforeThrow.throw(marker); } catch (error) { caught = error; }
checkMarker(caught);
check(groupedPropertyBeforeThrow.next(), undefined, true);
const groupedPropertyBeforeReturn = groupedPropertyPending();
check(groupedPropertyBeforeReturn.next(), 'grouped-property-key', false);
check(groupedPropertyBeforeReturn.return(91), 91, true);
check(groupedPropertyBeforeReturn.next(), undefined, true);
if (groupedPropertyGets !== 0 || groupedPropertyLaterEffects !== 0) throw 'grouped Property abrupt key bypasses Get and outer operands';

const groupedPropertyAfterThrow = groupedPropertyPending();
check(groupedPropertyAfterThrow.next(), 'grouped-property-key', false);
check(groupedPropertyAfterThrow.next('method'), 'grouped-property-outer', false);
caught = undefined;
try { groupedPropertyAfterThrow.throw(marker); } catch (error) { caught = error; }
checkMarker(caught);
check(groupedPropertyAfterThrow.next(), undefined, true);
const groupedPropertyAfterReturn = groupedPropertyPending();
check(groupedPropertyAfterReturn.next(), 'grouped-property-key', false);
check(groupedPropertyAfterReturn.next('method'), 'grouped-property-outer', false);
check(groupedPropertyAfterReturn.return(92), 92, true);
check(groupedPropertyAfterReturn.next(), undefined, true);
if (groupedPropertyGets !== 2 || groupedPropertyCalls !== 0 || groupedPropertyLaterEffects !== 0) throw 'grouped Property abrupt outer bypasses Call';
const groupedPropertyActualThrow = groupedPropertyPending();
check(groupedPropertyActualThrow.next(), 'grouped-property-key', false);
check(groupedPropertyActualThrow.next('method'), 'grouped-property-outer', false);
caught = undefined;
try { groupedPropertyActualThrow.next(2); } catch (error) { caught = error; }
checkMarker(caught);
if (groupedPropertyAbruptTrace.join(',') !== 'finally,finally,get,finally,get,finally,get,last,call,finally') throw 'grouped Property injected completion order';

const groupedPropertyThrowingGet = {get method() { groupedPropertyAbruptTrace.push('throwing-get'); throw marker; }};
function* groupedPropertyGetterFailure() {
  try { const result = (groupedPropertyThrowingGet?.[yield 'grouped-property-get-key'])(yield forbidden()); assigned = result; }
  finally { groupedPropertyAbruptTrace.push('get-finally'); }
}
const groupedPropertyGetterIterator = groupedPropertyGetterFailure();
check(groupedPropertyGetterIterator.next(), 'grouped-property-get-key', false);
caught = undefined;
try { groupedPropertyGetterIterator.next('method'); } catch (error) { caught = error; }
checkMarker(caught);
const groupedPropertyThrowingKey = {[Symbol.toPrimitive]() { groupedPropertyAbruptTrace.push('key-throw'); throw marker; }};
function* groupedPropertyKeyFailure() {
  try { const result = (groupedPropertyAbruptOwner?.[yield 'grouped-property-coerce-key'])(yield forbidden()); assigned = result; }
  finally { groupedPropertyAbruptTrace.push('key-finally'); }
}
const groupedPropertyKeyIterator = groupedPropertyKeyFailure();
check(groupedPropertyKeyIterator.next(), 'grouped-property-coerce-key', false);
caught = undefined;
try { groupedPropertyKeyIterator.next(groupedPropertyThrowingKey); } catch (error) { caught = error; }
checkMarker(caught);
if (groupedPropertyGets !== 3 || later !== 0) throw 'grouped Property abrupt Get/coercion stops outer operands';

const groupedPropertyThrowingIterable = {[Symbol.iterator]() {
  groupedPropertyAbruptTrace.push('iterator');
  return {next() {
    return {get done() { groupedPropertyAbruptTrace.push('done'); return false; }, get value() { groupedPropertyAbruptTrace.push('value'); throw marker; }};
  }, return() { groupedPropertyCloses++; throw 'grouped Property argument IteratorClose'; }};
}};
function* groupedPropertySpreadFailure() {
  try { const result = (groupedPropertyAbruptOwner?.[yield 'grouped-property-spread-key'])(...groupedPropertyThrowingIterable, yield forbidden()); assigned = result; }
  finally { groupedPropertyAbruptTrace.push('spread-finally'); }
}
const groupedPropertySpreadIterator = groupedPropertySpreadFailure();
check(groupedPropertySpreadIterator.next(), 'grouped-property-spread-key', false);
caught = undefined;
try { groupedPropertySpreadIterator.next('method'); } catch (error) { caught = error; }
checkMarker(caught);

const groupedPropertyTagFailureOwner = {get tag() {
  groupedPropertyAbruptTrace.push('tag-get');
  return function(template, value) {
    'use strict';
    if (this !== groupedPropertyTagFailureOwner || value !== 4 || !Object.isFrozen(template) || !Object.isFrozen(template.raw)) throw 'grouped Property abrupt tag Reference';
    groupedPropertyAbruptTrace.push('tag-call');
    throw marker;
  };
}};
function* groupedPropertyTagFailure() {
  try { const result = (groupedPropertyTagFailureOwner?.[yield 'grouped-property-tag-key'])`head${yield 'grouped-property-tag-value'}tail`; assigned = result; }
  finally { groupedPropertyAbruptTrace.push('tag-finally'); }
}
const groupedPropertyTagIterator = groupedPropertyTagFailure();
check(groupedPropertyTagIterator.next(), 'grouped-property-tag-key', false);
check(groupedPropertyTagIterator.next('tag'), 'grouped-property-tag-value', false);
caught = undefined;
try { groupedPropertyTagIterator.next(4); } catch (error) { caught = error; }
checkMarker(caught);

const groupedPropertyNoncallable = {method: 0};
function* groupedPropertyIntrinsicFailure() {
  try { const result = (groupedPropertyNoncallable?.[yield 'grouped-property-invalid-key'])(yield 'grouped-property-invalid-value'); assigned = result; }
  finally { groupedPropertyAbruptTrace.push('invalid-finally'); }
}
const groupedPropertyInvalidIterator = groupedPropertyIntrinsicFailure();
check(groupedPropertyInvalidIterator.next(), 'grouped-property-invalid-key', false);
check(groupedPropertyInvalidIterator.next('method'), 'grouped-property-invalid-value', false);
caught = undefined;
try { groupedPropertyInvalidIterator.next(5); } catch (error) { caught = error; }
if (Object.getPrototypeOf(caught) !== intrinsicPrototype || assigned !== 'kept') throw 'grouped Property generated intrinsic Realm after public binding mutation';
if (groupedPropertyGets !== 4 || groupedPropertyCalls !== 1 || groupedPropertyLaterEffects !== 1 || groupedPropertyCloses !== 0 || later !== 0) throw 'grouped Property abrupt observation counts';
if (groupedPropertyAbruptTrace.join(',') !== 'finally,finally,get,finally,get,finally,get,last,call,finally,throwing-get,get-finally,key-throw,key-finally,get,iterator,done,value,spread-finally,tag-get,tag-call,tag-finally,invalid-finally') throw 'grouped Property abrupt identity and finally order';
print('generator-optional-abrupt:ok');
262;
