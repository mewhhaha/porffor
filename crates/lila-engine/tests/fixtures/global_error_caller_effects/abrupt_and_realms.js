function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'public constructor or unreachable phase'; }
const define = Object.defineProperty;
const descriptor = Object.getOwnPropertyDescriptor;
const getPrototypeOf = Object.getPrototypeOf;
const own = Object.prototype.hasOwnProperty;
const LocalError = Error;
const LocalAggregate = AggregateError;
const LocalSuppressed = SuppressedError;
const localErrorPrototype = Error.prototype;
const localTypePrototype = TypeError.prototype;
const localArrayPrototype = Array.prototype;
const localToString = Error.prototype.toString;
const foreign = __lilaCreateRealm().global;
const ForeignError = foreign.Error;
const ForeignAggregate = foreign.AggregateError;
const ForeignObject = foreign.Object;
const foreignBoundObject = foreign.Function.prototype.bind.call(ForeignObject, null);
const foreignErrorPrototype = foreign.Error.prototype;
const foreignTypePrototype = foreign.TypeError.prototype;
const foreignArrayPrototype = foreign.Array.prototype;
const foreignToString = foreign.Error.prototype.toString;
const marker = new ForeignError('Global/Error original marker');
const localErrorDescriptor = descriptor(globalThis, 'Error');
const localTypeDescriptor = descriptor(globalThis, 'TypeError');
const localArrayDescriptor = descriptor(globalThis, 'Array');
const foreignErrorDescriptor = descriptor(foreign, 'Error');
const foreignTypeDescriptor = descriptor(foreign, 'TypeError');
const foreignArrayDescriptor = descriptor(foreign, 'Array');
function restore(target, key, original) {
  if (original === undefined) delete target[key];
  else define(target, key, original);
}
function expectMarker(action, expected, trace) {
  let result = 'unpublished', caught;
  try { const value = action(); result = value; }
  catch (error) { caught = error; trace.push('catch'); }
  finally { trace.push('finally'); }
  check(caught === marker && getPrototypeOf(caught) === foreignErrorPrototype && result === 'unpublished' &&
    trace.join(',') === expected + ',catch,finally', 'original foreign abrupt value, phase cutoff and finally');
}
function expectNative(action, prototype) {
  let caught, finalized = 0;
  try { action(); } catch (error) { caught = error; } finally { finalized++; }
  check(caught !== undefined && getPrototypeOf(caught) === prototype && finalized === 1,
    'native TypeError uses the called method Realm');
}

try {
  const ignoredTrace = [];
  const unusedInput = {toString() { ignoredTrace.push('unreached.coerce'); return 'A B'; }};
  function ignoredFailure() { ignoredTrace.push('extra'); throw marker; }
  expectMarker(() => encodeURI((ignoredTrace.push('first'), unusedInput), ignoredFailure()),
    'first,extra', ignoredTrace);

  const codecTrace = [];
  let codecKind = 1;
  expectMarker(() => encodeURIComponent({toString() {
    codecTrace.push('coerce'); codecKind = function() { return 7; }; throw marker;
  }}), 'coerce', codecTrace);
  check(typeof codecKind === 'function' && codecKind() === 7, 'codec hook prior write survives throw');

  const prototypeTrace = [];
  let prototypeKind = 1;
  const badNewTarget = new Proxy(function() {}, {get(target, key, rawThis) {
    if (key === 'prototype') {
      prototypeTrace.push('prototype'); prototypeKind = function() { return 7; }; throw marker;
    }
    return Reflect.get(target, key, rawThis);
  }});
  const unusedMessage = {toString() { prototypeTrace.push('unreached.message'); return 'message'; }};
  const unusedOptions = new Proxy({}, {has() { prototypeTrace.push('unreached.has'); return true; }, get: poison});
  expectMarker(() => Reflect.construct(LocalError, [unusedMessage, unusedOptions], badNewTarget),
    'prototype', prototypeTrace);
  check(typeof prototypeKind === 'function' && prototypeKind() === 7, 'NewTarget hook prior write survives throw');

  const messageTrace = [];
  let messageShape = {value: 1};
  const messageOptions = new Proxy({}, {has() { messageTrace.push('unreached.has'); return true; }, get: poison});
  expectMarker(() => Error({toString() {
    messageTrace.push('message'); messageShape = {value: Symbol.for('message cutoff')}; throw marker;
  }}, messageOptions), 'message', messageTrace);
  check(typeof messageShape.value === 'symbol', 'message prior shape write survives throw');

  const hasTrace = [];
  let hasKind = 1;
  const badHas = new Proxy({}, {has(target, key) {
    check(key === 'cause', 'cause HasProperty key'); hasTrace.push('has');
    hasKind = function() { return 7; }; throw marker;
  }, get() { hasTrace.push('unreached.get'); throw marker; }});
  expectMarker(() => Error({toString() { hasTrace.push('message'); return 'message'; }}, badHas),
    'message,has', hasTrace);
  check(typeof hasKind === 'function' && hasKind() === 7, 'cause Has prior write survives throw');

  const getTrace = [];
  const getElements = [1];
  const badGet = new Proxy({}, {
    has(target, key) { check(key === 'cause', 'cause Has key'); getTrace.push('has'); return true; },
    get(target, key, rawThis) {
      check(key === 'cause' && rawThis === badGet, 'cause Get original receiver');
      getTrace.push('get'); getElements[0] = function() { return 9; }; throw marker;
    }
  });
  expectMarker(() => Error({toString() { getTrace.push('message'); return 'message'; }}, badGet),
    'message,has,get', getTrace);
  check(typeof getElements[0] === 'function' && getElements[0]() === 9, 'cause Get prior element write survives throw');

  const aggregatePrefix = [];
  const unvisitedErrors = {get [Symbol.iterator]() { aggregatePrefix.push('unreached.iterator'); throw marker; }};
  const aggregateOptions = new Proxy({}, {
    has(target, key) { check(key === 'cause', 'Aggregate prefix Has key'); aggregatePrefix.push('has'); return true; },
    get(target, key) { check(key === 'cause', 'Aggregate prefix Get key'); aggregatePrefix.push('get'); throw marker; }
  });
  expectMarker(() => AggregateError(unvisitedErrors, {toString() { aggregatePrefix.push('message'); return 'message'; }},
    aggregateOptions), 'message,has,get', aggregatePrefix);

  let aggregateCloses = 0;
  for (let stage = 0; stage < 3; stage++) {
    const trace = [];
    const iterator = {get next() {
      trace.push('next.get');
      return function() {
        trace.push('next'); if (stage === 0) throw marker;
        return {get done() { trace.push('done'); if (stage === 1) throw marker; return false; },
          get value() { trace.push('value'); throw marker; }};
      };
    }, get return() { aggregateCloses++; throw 'Aggregate abrupt list has no close'; }};
    const errors = {get [Symbol.iterator]() {
      trace.push('iterator.get'); return function() { trace.push('iterator.call'); return iterator; };
    }};
    const expected = stage === 0 ? 'iterator.get,iterator.call,next.get,next' :
      stage === 1 ? 'iterator.get,iterator.call,next.get,next,done' : 'iterator.get,iterator.call,next.get,next,done,value';
    expectMarker(() => AggregateError(errors), expected, trace);
  }
  check(aggregateCloses === 0, 'Aggregate next/done/value abrupt completions never Get return');

  const suppressedTrace = [];
  let suppressedKind = 1;
  const rawError = {toString() { suppressedTrace.push('unreached.error'); throw marker; }};
  const rawSuppressed = {toString() { suppressedTrace.push('unreached.suppressed'); throw marker; }};
  const noCause = new Proxy({}, {has() { suppressedTrace.push('unreached.has'); throw marker; }, get: poison});
  expectMarker(() => SuppressedError(rawError, rawSuppressed, {toString() {
    suppressedTrace.push('message'); suppressedKind = function() { return 7; }; throw marker;
  }}, noCause), 'message', suppressedTrace);
  check(typeof suppressedKind === 'function' && suppressedKind() === 7, 'Suppressed third-message prior write survives');

  for (let stage = 0; stage < 4; stage++) {
    const trace = [], receiver = {};
    define(receiver, 'name', {get() {
      trace.push('name.get'); if (stage === 0) throw marker;
      return {toString() { trace.push('name.string'); if (stage === 1) throw marker; return 'Name'; }};
    }});
    define(receiver, 'message', {get() {
      trace.push('message.get'); if (stage === 2) throw marker;
      return {toString() { trace.push('message.string'); throw marker; }};
    }});
    const expected = stage === 0 ? 'name.get' : stage === 1 ? 'name.get,name.string' :
      stage === 2 ? 'name.get,name.string,message.get' : 'name.get,name.string,message.get,message.string';
    expectMarker(() => localToString.call(receiver), expected, trace);
  }

  let spreadCloses = 0, constructed = 0;
  const spreadTrace = [];
  const acquired = {saved: new Proxy(LocalError, {apply() { constructed++; throw 'callee after spread failure'; }})};
  const spreadIterator = {next() {
    spreadTrace.push('next'); return {get done() { spreadTrace.push('done'); return false; },
      get value() { spreadTrace.push('value'); throw marker; }};
  }, get return() { spreadCloses++; throw 'argument spread has no close'; }};
  const spread = {[Symbol.iterator]() { spreadTrace.push('iterator'); return spreadIterator; }};
  expectMarker(() => acquired.saved(...spread, (spreadTrace.push('unreached.extra'), 7)),
    'iterator,next,done,value', spreadTrace);
  check(spreadCloses === 0 && constructed === 0, 'spread abrupt prevents constructor and ignored operand without close');

  // Save both Realm authorities before poisoning their public bindings.
  globalThis.Error = poison; globalThis.TypeError = poison; globalThis.Array = poison;
  foreign.Error = poison; foreign.TypeError = poison; foreign.Array = poison;
  const localObject = LocalError.call(new ForeignObject(), 'local');
  const foreignObject = ForeignError.call({}, 'foreign');
  check(getPrototypeOf(localObject) === localErrorPrototype && localObject.message === 'local' &&
    getPrototypeOf(foreignObject) === foreignErrorPrototype && foreignObject.message === 'foreign',
    'Error calls use called constructor Realm despite raw opposite-Realm receiver and public poison');
  const localAggregate = LocalAggregate.call(new ForeignObject(), [marker], 'local aggregate');
  const foreignAggregate = ForeignAggregate.call({}, [marker], 'foreign aggregate');
  check(getPrototypeOf(localAggregate.errors) === localArrayPrototype && localAggregate.errors[0] === marker &&
    getPrototypeOf(foreignAggregate.errors) === foreignArrayPrototype && foreignAggregate.errors[0] === marker,
    'Aggregate errors arrays use called constructor Realm');
  const localSuppressed = LocalSuppressed(marker, foreignObject, 'suppressed');
  check(localSuppressed.error === marker && localSuppressed.suppressed === foreignObject &&
    !own.call(localSuppressed, 'cause'), 'Suppressed stores foreign original values without public Error lookup');

  // A bound constructor has no own non-writable prototype for the Proxy to contradict.
  const oppositeTarget = new Proxy(foreignBoundObject, {get(target, key, rawThis) {
    if (key === 'prototype') return undefined;
    return Reflect.get(target, key, rawThis);
  }});
  const constructedForeign = Reflect.construct(LocalError, ['opposite NewTarget'], oppositeTarget);
  check(getPrototypeOf(constructedForeign) === foreignErrorPrototype,
    'NewTarget Realm chooses default Error prototype independently of called constructor');
  check(localToString.call(new ForeignObject()) === 'Error' && foreignToString.call({}) === 'Error',
    'borrowed Error.toString observes opposite-Realm objects');
  expectNative(() => foreignToString.call(1), foreignTypePrototype);
  expectNative(() => localToString.call(foreign.Symbol('raw receiver')), localTypePrototype);
} finally {
  restore(globalThis, 'Error', localErrorDescriptor);
  restore(globalThis, 'TypeError', localTypeDescriptor);
  restore(globalThis, 'Array', localArrayDescriptor);
  restore(foreign, 'Error', foreignErrorDescriptor);
  restore(foreign, 'TypeError', foreignTypeDescriptor);
  restore(foreign, 'Array', foreignArrayDescriptor);
}
print('global-error-abrupt:ok');
262;
