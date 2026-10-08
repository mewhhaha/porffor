// Fresh literal arrays with default prototypes reach the retiring early gates.
const numberJoin = [1, 2];
numberJoin.join = function() {
  'use strict';
  if (this !== numberJoin || arguments.length !== 0) throw 'own join raw receiver';
  return 17;
};
const joinedNumber = numberJoin.join();
if (typeof joinedNumber !== 'number' || joinedNumber + 1 !== 18) throw 'own join Number result';
const ownStringMarker = {value: 'own-toString'};
const ownString = [1, 2];
ownString.toString = function() {
  'use strict';
  if (this !== ownString || arguments.length !== 0) throw 'own toString raw receiver';
  return ownStringMarker;
};
const ownStringResult = ownString.toString();
if (ownStringResult !== ownStringMarker || ownStringResult.value !== 'own-toString') throw 'own toString Object result';
function reverseMarker() { return 23; }
const ownReverse = [1, 2];
ownReverse.reverse = function() {
  'use strict';
  if (this !== ownReverse || arguments.length !== 0) throw 'own reverse raw receiver';
  return reverseMarker;
};
const ownReverseResult = ownReverse.reverse();
if (typeof ownReverseResult !== 'function' || ownReverseResult !== reverseMarker || ownReverseResult() !== 23) throw 'own reverse Function result';

const sharedString = [1, 2];
function sharedMarker() { return 29; }
sharedString.join = function() {
  if (this !== sharedString || arguments.length !== 0) throw 'shared toString join Call';
  return sharedMarker;
};
const sharedResult = sharedString.toString();
if (typeof sharedResult !== 'function' || sharedResult !== sharedMarker || sharedResult() !== 29) throw 'shared toString arbitrary join result';
const fallback = [];
fallback.join = 0;
if (fallback.toString() !== '[object Array]') throw 'noncallable join intrinsic fallback';
const freshReverse = [1, 'x'];
const reversed = freshReverse.reverse();
if (reversed !== freshReverse || typeof reversed[0] !== 'string' || reversed[0].charAt(0) !== 'x' ||
    typeof freshReverse[1] !== 'number' || freshReverse[1] + 1 !== 2) throw 'reverse result and caller live facts';

function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'replaced callee must not be called'; }
const define = Object.defineProperty;
const nativeJoin = Array.prototype.join;
const nativeReverse = Array.prototype.reverse;
const proxyArray = [1, 2];
let proxyCalls = 0;
const proxyTarget = function() { throw 'Proxy target bypass'; };
proxyArray.join = new Proxy(proxyTarget, {apply(target, receiver, args) {
  check(target === proxyTarget && receiver === proxyArray && args.length === 1 && args[0] === ':', 'own Proxy join Call');
  proxyCalls++; return 31;
}});
const proxyNumber = proxyArray.join(':');
check(typeof proxyNumber === 'number' && proxyNumber + 1 === 32 && proxyCalls === 1, 'own Proxy join arbitrary result');

const trace = [];
const original = ['a', 'b'];
let selected = original;
define(original, 'join', {configurable: true, get() { trace.push('callee'); return nativeJoin; }});
function base() { trace.push('base'); return selected; }
function separatorArgument() {
  trace.push('separator.arg');
  selected = ['wrong'];
  define(original, 'join', {value: poison});
  return {toString() { trace.push('separator.coerce'); return ':'; }};
}
const iterator = {};
let position = 0;
const next = new Proxy(function() {}, {apply(target, receiver, args) {
  check(receiver === iterator && args.length === 0, 'argument spread next Call');
  trace.push('next');
  const index = position++;
  check(index < 2, 'bounded argument spread');
  if (index === 0) define(iterator, 'next', {value: poison});
  return {
    get done() { trace.push('done' + index); return index === 1; },
    get value() { check(index === 0, 'terminal value unread'); trace.push('value0'); return 'ignored'; }
  };
}});
define(iterator, 'next', {configurable: true, get() { trace.push('next.get'); return next; }});
define(iterator, 'return', {get() { throw 'exhausted argument spread must not close'; }});
const spread = [99];
define(spread, '0', {get() { throw 'numeric spread snapshot'; }});
define(spread, Symbol.iterator, {get() {
  trace.push('iterator.get');
  return new Proxy(function() {}, {apply(target, receiver, args) {
    check(receiver === spread && args.length === 0, 'argument spread iterator Call');
    trace.push('open'); return iterator;
  }});
}});
function tail() { trace.push('tail'); return 'also ignored'; }
check(base().join(separatorArgument(), ...spread, tail()) === 'a:b' && selected !== original, 'direct syntax captured before arguments');
check(trace.join(',') === 'base,callee,separator.arg,iterator.get,open,next.get,next,done0,value0,next,done1,tail,separator.coerce',
  'complete arguments precede join coercion');

const extras = [];
const reverseExtras = [1, 2];
reverseExtras.reverse = function(first, second) {
  'use strict';
  check(this === reverseExtras && arguments.length === 2 && first === 'first' && second === 'second', 'own reverse complete args');
  extras.push('call'); return {value: 37};
};
function extra(value) { extras.push(value); return value; }
const extraResult = reverseExtras.reverse(extra('first'), extra('second'));
check(extraResult.value === 37 && extras.join(',') === 'first,second,call', 'ignored native operands remain real for override');

const flow = {value: 1};
let lower = 1;
let upper = 'r';
const liveReverse = [1, 'r'];
const reverseTrace = [];
define(liveReverse, '0', {configurable: true, get() {
  reverseTrace.push('get0'); flow.value = 'getter'; return lower;
}, set(value) { reverseTrace.push('set0'); lower = value; }});
define(liveReverse, '1', {configurable: true, get() { reverseTrace.push('get1'); return upper; },
  set(value) { reverseTrace.push('set1'); upper = value; }});
check(liveReverse.reverse() === liveReverse && lower === 'r' && upper === 1, 'native reverse live accessors');
check(reverseTrace.join(',') === 'get0,get1,set0,set1' && typeof flow.value === 'string' && flow.value.charAt(0) === 'g',
  'reverse order and getter invalidation');

const foreign = $262.createRealm().global;
const LocalArray = Array;
const ForeignArray = foreign.Array;
const getPrototypeOf = Object.getPrototypeOf;
const localTypeErrorPrototype = TypeError.prototype;
const foreignTypeErrorPrototype = foreign.TypeError.prototype;
const foreignReverse = ForeignArray.prototype.reverse;
const marker = new foreign.Error('array shortcut marker');
TypeError = poison; foreign.TypeError = poison;
function reverseError(ctor, method, errorPrototype) {
  const source = new ctor(2);
  source[0] = 1; source[1] = 2;
  source.reverse = method;
  define(source, '0', {writable: false});
  let caught;
  let finalized = 0;
  try { source.reverse(); } catch (error) { caught = error; } finally { finalized++; }
  check(caught !== undefined && getPrototypeOf(caught) === errorPrototype && finalized === 1 && source[0] === 1 && source[1] === 2,
    'borrowed reverse native TypeError defining Realm');
}
reverseError(ForeignArray, nativeReverse, localTypeErrorPrototype);
reverseError(LocalArray, foreignReverse, foreignTypeErrorPrototype);

const abruptTrace = [];
const inaccessible = [1];
define(inaccessible, 'join', {get() { abruptTrace.push('callee'); throw marker; }});
let caught;
let result = 'unpublished';
let prior = 17;
function unreachableArgument() { throw 'argument after callee Get throw'; }
try { prior = 18; result = inaccessible.join(unreachableArgument()); }
catch (error) { caught = error; }
finally { abruptTrace.push('finally'); }
check(caught === marker && result === 'unpublished' && prior === 18 && abruptTrace.join(',') === 'callee,finally', 'own acquisition original abrupt');
const coercionSource = [1];
define(coercionSource, '0', {get() { throw 'element after separator throw'; }});
caught = undefined; result = 'unpublished';
const separator = {toString() { throw marker; }};
let finalized = 0;
try { result = coercionSource.join(separator); }
catch (error) { caught = error; }
finally { finalized++; }
check(caught === marker && result === 'unpublished' && finalized === 1, 'native join coercion original abrupt');
const argumentSource = [1];
define(argumentSource, '0', {get() { throw 'element before arguments complete'; }});
function failingArgument() { throw marker; }
caught = undefined; result = 'unpublished';
try { result = argumentSource.join(failingArgument()); } catch (error) { caught = error; }
check(caught === marker && result === 'unpublished', 'direct join argument throw skips native body');
let closes = 0;
const failingIterator = {next() { return {get done() { return false; }, get value() { throw marker; }}; }};
define(failingIterator, 'return', {get() { closes++; throw 'direct join spread must not close'; }});
const failingSpread = {[Symbol.iterator]() { return failingIterator; }};
caught = undefined; result = 'unpublished';
try { result = argumentSource.join(...failingSpread); } catch (error) { caught = error; }
check(caught === marker && result === 'unpublished' && closes === 0, 'direct join spread failure skips native body');
const throwingString = [1];
throwingString.join = function() { throw marker; };
caught = undefined;
try { throwingString.toString(); } catch (error) { caught = error; }
check(caught === marker, 'shared toString original join throw');
print('invocation-shortcut-arrays:ok');
262;
