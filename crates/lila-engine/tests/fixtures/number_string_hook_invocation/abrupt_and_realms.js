function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'public constructor, reacquired method or unreachable operand'; }
const numberPrototype = Number.prototype;
const matchDescriptor = Object.getOwnPropertyDescriptor(numberPrototype, 'match');
const splitDescriptor = Object.getOwnPropertyDescriptor(numberPrototype, 'split');
const define = Object.defineProperty;
const getPrototypeOf = Object.getPrototypeOf;
const nativeMatch = String.prototype.match;
const nativeSplit = String.prototype.split;
const localTypePrototype = TypeError.prototype;
const foreign = $262.createRealm().global;
const foreignMatch = foreign.String.prototype.match;
const foreignSplit = foreign.String.prototype.split;
const ForeignNumber = foreign.Number;
const foreignTypePrototype = foreign.TypeError.prototype;
const foreignErrorPrototype = foreign.Error.prototype;
const marker = new foreign.Error('Number borrowed String-hook marker');
function restore(key, descriptor) {
  if (descriptor === undefined) delete numberPrototype[key];
  else define(numberPrototype, key, descriptor);
}
function expectMarker(action, expected, trace) {
  let result = 'unpublished', caught;
  try { const value = action(); result = value; }
  catch (error) { caught = error; trace.push('catch'); }
  finally { trace.push('finally'); }
  check(caught === marker && getPrototypeOf(caught) === foreignErrorPrototype && result === 'unpublished' &&
    trace.join(',') === expected + ',catch,finally', 'original foreign abrupt identity and precise cutoff');
}
function expectNative(action, prototype) {
  let caught, finalized = 0;
  try { action(); } catch (error) { caught = error; } finally { finalized++; }
  check(caught !== undefined && getPrototypeOf(caught) === prototype && finalized === 1,
    'native TypeError belongs to called String method Realm');
}
try {
  const acquisitionTrace = [];
  define(numberPrototype, 'match', {configurable: true, get() { acquisitionTrace.push('callee'); throw marker; }});
  expectMarker(() => (123).match((acquisitionTrace.push('unreached.argument'), 1)), 'callee', acquisitionTrace);
  define(numberPrototype, 'match', {configurable: true, writable: true, value: nativeMatch});

  Number.prototype.match = String.prototype.match;
  const firstTrace = [];
  function failingFirst() { firstTrace.push('first.argument'); throw marker; }
  expectMarker(() => (123).match(failingFirst()), 'first.argument', firstTrace);

  Number.prototype.split = String.prototype.split;
  const secondTrace = [];
  // Identifier + this exact single-return function expression reaches the
  // retired recognized-separator gate without an unsupported comma shape.
  var recognizedSeparator = {toString: function() { return /x/; }, [Symbol.split]() {
    secondTrace.push('unreached.hook'); return 7;
  }};
  function failingSecond() { secondTrace.push('second.argument'); throw marker; }
  expectMarker(() => (123).split(recognizedSeparator, failingSecond()), 'second.argument', secondTrace);

  const limitTrace = [];
  var recognizedFallback = {toString: function() { return /x/; }, valueOf() {
    limitTrace.push('unreached.separator'); throw marker;
  }};
  define(recognizedFallback, Symbol.split, {get() { limitTrace.push('hook.get'); return undefined; }});
  const failingLimit = {valueOf() { limitTrace.push('limit.coerce'); throw marker; }};
  function limitArgument() { limitTrace.push('limit.argument'); return failingLimit; }
  expectMarker(() => (123).split(recognizedFallback, limitArgument()),
    'limit.argument,hook.get,limit.coerce', limitTrace);

  const separatorTrace = [];
  var recognizedPrimitiveFailure = {toString: function() { return /x/; }, valueOf() {
    separatorTrace.push('separator.valueOf'); throw marker;
  }};
  define(recognizedPrimitiveFailure, Symbol.split, {get() { separatorTrace.push('hook.get'); return undefined; }});
  const finiteLimit = {valueOf() { separatorTrace.push('limit.coerce'); return 2; }};
  expectMarker(() => (123).split(recognizedPrimitiveFailure, finiteLimit),
    'hook.get,limit.coerce,separator.valueOf', separatorTrace);

  const hookGetTrace = [];
  const getPattern = {};
  define(getPattern, Symbol.match, {get() { hookGetTrace.push('hook.get'); throw marker; }});
  expectMarker(() => (123).match((hookGetTrace.push('argument'), getPattern)), 'argument,hook.get', hookGetTrace);
  const hookCallTrace = [];
  const callPattern = {};
  define(callPattern, Symbol.split, {get() {
    hookCallTrace.push('hook.get');
    return new Proxy(function() {}, {apply(target, receiver, args) {
      check(receiver === callPattern && args.length === 2 && args[0] === 123 && args[1] === 2,
        'abrupt symbol Proxy receives original primitive operands');
      hookCallTrace.push('apply'); throw marker;
    }});
  }});
  expectMarker(() => (123).split((hookCallTrace.push('first.argument'), callPattern),
    (hookCallTrace.push('second.argument'), 2)), 'first.argument,second.argument,hook.get,apply', hookCallTrace);

  let returnGets = 0;
  for (let stage = 0; stage < 6; stage++) {
    const trace = [];
    const iterator = {};
    define(iterator, 'return', {get() { returnGets++; throw 'argument spread close'; }});
    define(iterator, 'next', {get() {
      trace.push('next.get'); if (stage === 2) throw marker;
      return new Proxy(function() {}, {apply(target, receiver, args) {
        check(receiver === iterator && args.length === 0, 'abrupt cached next Reference');
        trace.push('next'); if (stage === 3) throw marker;
        return {get done() { trace.push('done'); if (stage === 4) throw marker; return false; },
          get value() { trace.push('value'); throw marker; }};
      }});
    }});
    const iterable = {};
    define(iterable, Symbol.iterator, {get() {
      trace.push('iterator.get'); if (stage === 0) throw marker;
      return new Proxy(function() {}, {apply(target, receiver, args) {
        check(receiver === iterable && args.length === 0, 'abrupt iterator acquisition Reference');
        trace.push('open'); if (stage === 1) throw marker; return iterator;
      }});
    }});
    const expected = ['iterator.get', 'iterator.get,open', 'iterator.get,open,next.get',
      'iterator.get,open,next.get,next', 'iterator.get,open,next.get,next,done',
      'iterator.get,open,next.get,next,done,value'][stage];
    expectMarker(() => (123).split(...iterable, (trace.push('unreached.argument'), 2)), expected, trace);
  }
  check(returnGets === 0, 'no IteratorClose or return Get after any spread acquisition/step abrupt');

  // Saved intrinsic identities survive poisoning public error/constructor names.
  globalThis.String = poison; foreign.String = poison;
  globalThis.TypeError = poison; foreign.TypeError = poison;
  const badMatch = {[Symbol.match]: 0};
  const badSplit = {[Symbol.split]: 0};
  Number.prototype.match = foreignMatch;
  Number.prototype.split = foreignSplit;
  expectNative(() => (123).match(badMatch), foreignTypePrototype);
  expectNative(() => (123).split(badSplit, 2), foreignTypePrototype);
  const foreignReceiver = new ForeignNumber(123);
  foreignReceiver.match = nativeMatch;
  foreignReceiver.split = nativeSplit;
  expectNative(() => foreignReceiver.match(badMatch), localTypePrototype);
  expectNative(() => foreignReceiver.split(badSplit, 2), localTypePrototype);
} finally {
  restore('match', matchDescriptor);
  restore('split', splitDescriptor);
}
check(Object.getOwnPropertyDescriptor(numberPrototype, 'match') === undefined ? matchDescriptor === undefined :
  Object.getOwnPropertyDescriptor(numberPrototype, 'match').value === matchDescriptor.value, 'match descriptor restored');
check(Object.getOwnPropertyDescriptor(numberPrototype, 'split') === undefined ? splitDescriptor === undefined :
  Object.getOwnPropertyDescriptor(numberPrototype, 'split').value === splitDescriptor.value, 'split descriptor restored');
print('number-string-hooks-abrupt:ok');
262;
