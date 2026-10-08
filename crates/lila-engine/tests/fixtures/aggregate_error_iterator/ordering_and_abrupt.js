const other = __lilaCreateRealm().global;
const LocalAggregateError = AggregateError;
const ForeignAggregateError = other.AggregateError;
const localArrayPrototype = Array.prototype;
const foreignArrayPrototype = other.Array.prototype;
const localTypePrototype = TypeError.prototype;
const foreignTypePrototype = other.TypeError.prototype;
const ForeignError = other.Error;
const marker = new ForeignError('aggregate-iterator-marker');
function wrong() { throw 'public constructor observed'; }
globalThis.AggregateError = wrong; other.AggregateError = wrong;
globalThis.Array = wrong; other.Array = wrong;
globalThis.TypeError = wrong; other.TypeError = wrong;

function expectMarker(action, trace, expected) {
  const previous = {};
  let result = previous;
  let caught = false;
  try { result = action(); } catch (error) {
    if (error !== marker || Object.getPrototypeOf(error) !== ForeignError.prototype) throw 'original foreign identity';
    caught = true; trace.push('catch');
  } finally { trace.push('finally'); }
  if (!caught || result !== previous || trace.join(',') !== expected) throw 'unpublished abrupt result';
}
function invoke(Constructor, input, construct) {
  return construct ? new Constructor(input) : Constructor(input);
}

let nativeCases = 0;
let markerCases = 0;
for (let direction = 0; direction < 2; direction++) {
  const Constructor = direction === 0 ? LocalAggregateError : ForeignAggregateError;
  const typePrototype = direction === 0 ? localTypePrototype : foreignTypePrototype;
  const arrayPrototype = direction === 0 ? localArrayPrototype : foreignArrayPrototype;
  const OtherNewTarget = direction === 0 ? ForeignAggregateError : LocalAggregateError;
  const explicitPrototype = {direction};
  const prefix = [];
  const newTarget = new Proxy(function target() { throw 'NewTarget body'; }, {
    get(target, key, receiver) {
      if (key === 'prototype') { prefix.push('prototype'); return explicitPrototype; }
      return Reflect.get(target, key, receiver);
    }
  });
  const message = {[Symbol.toPrimitive](hint) {
    if (hint !== 'string') throw 'message hint'; prefix.push('message'); return 'message-value';
  }};
  const options = {get cause() { prefix.push('cause'); return marker; }};
  const iterator = {get next() {
    prefix.push('next.get');
    return function() {
      if (this !== iterator || arguments.length !== 0) throw 'prefix next invocation';
      prefix.push('next.call');
      return {get done() { prefix.push('done'); return true; }, get value() { throw 'terminal prefix value'; }};
    };
  }, get return() { throw 'prefix return Get'; }};
  const iterable = {get [Symbol.iterator]() {
    prefix.push('iterator.get');
    return function() {
      if (this !== iterable || arguments.length !== 0) throw 'prefix iterator invocation';
      prefix.push('iterator.call'); return iterator;
    };
  }};
  const result = Reflect.construct(Constructor, [iterable, message, options], newTarget);
  if (prefix.join(',') !== 'prototype,message,cause,iterator.get,iterator.call,next.get,next.call,done') throw 'constructor prefix before iterator';
  if (Object.getPrototypeOf(result) !== explicitPrototype || result.message !== 'message-value' || result.cause !== marker) throw 'NewTarget and prefix fields';
  if (result.errors.length !== 0 || Object.getPrototypeOf(result.errors) !== arrayPrototype) throw 'called-Realm list after prefix';

  for (const fault of ['prototype', 'message', 'cause']) {
    const trace = [];
    let iterableGets = 0;
    const earlyTarget = new Proxy(function target() {}, {get(target, key, receiver) {
      if (key === 'prototype') {
        trace.push('prototype'); if (fault === 'prototype') throw marker; return explicitPrototype;
      }
      return Reflect.get(target, key, receiver);
    }});
    const earlyMessage = {[Symbol.toPrimitive](hint) {
      if (hint !== 'string') throw 'early message hint';
      trace.push('message'); if (fault === 'message') throw marker; return 'ok';
    }};
    const earlyOptions = {get cause() {
      trace.push('cause'); if (fault === 'cause') throw marker; return 1;
    }};
    const unread = {get [Symbol.iterator]() { iterableGets++; throw 'iterable after prefix throw'; }};
    const expected = fault === 'prototype' ? 'prototype,catch,finally' :
      fault === 'message' ? 'prototype,message,catch,finally' : 'prototype,message,cause,catch,finally';
    expectMarker(() => Reflect.construct(Constructor, [unread, earlyMessage, earlyOptions], earlyTarget), trace, expected);
    if (iterableGets !== 0) throw 'prefix abrupt skips iterable';
    markerCases++;
  }

  for (let construct = 0; construct < 2; construct++) {
    for (const fault of ['iterator.get', 'iterator.apply', 'next.get', 'next.apply', 'done.get', 'value.get']) {
      const trace = [];
      let returnGets = 0;
      const stepTarget = function() { throw 'direct next target'; };
      const faultIterator = {get next() {
        trace.push('next.get'); if (fault === 'next.get') throw marker;
        return new Proxy(stepTarget, {apply(target, receiver, args) {
          if (target !== stepTarget || receiver !== faultIterator || args.length !== 0) throw 'abrupt next receiver/argc0';
          trace.push('next.apply'); if (fault === 'next.apply') throw marker;
          return {get done() { trace.push('done.get'); if (fault === 'done.get') throw marker; return false; },
            get value() { trace.push('value.get'); throw marker; }};
        }});
      }, get return() { returnGets++; throw 'IteratorClose on operation abrupt'; }};
      const methodTarget = function() { throw 'direct iterator target'; };
      const faultInput = {get [Symbol.iterator]() {
        trace.push('iterator.get'); if (fault === 'iterator.get') throw marker;
        return new Proxy(methodTarget, {apply(target, receiver, args) {
          if (target !== methodTarget || receiver !== faultInput || args.length !== 0) throw 'abrupt iterator receiver/argc0';
          trace.push('iterator.apply'); if (fault === 'iterator.apply') throw marker; return faultIterator;
        }});
      }};
      const operations = ['iterator.get', 'iterator.apply', 'next.get', 'next.apply', 'done.get', 'value.get'];
      let expected = '';
      for (const operation of operations) {
        expected += (expected === '' ? '' : ',') + operation;
        if (operation === fault) break;
      }
      expectMarker(() => invoke(Constructor, faultInput, construct), trace, expected + ',catch,finally');
      if (returnGets !== 0) throw 'operation abrupt never reads return';
      markerCases++;
    }

    for (const fault of ['missing', 'noncallable', 'method-result', 'next-noncallable', 'next-result']) {
      let returnGets = 0;
      let nextGets = 0;
      const protocolIterator = {get next() {
        nextGets++;
        return fault === 'next-noncallable' ? 1 : function() { return 1; };
      }, get return() { returnGets++; throw marker; }};
      const input = {get [Symbol.iterator]() {
        if (fault === 'missing') return undefined;
        if (fault === 'noncallable') return 1;
        return function() { return fault === 'method-result' ? 1 : protocolIterator; };
      }};
      const previous = {};
      let output = previous;
      let caught = false;
      try { output = invoke(Constructor, input, construct); } catch (error) {
        if (Object.getPrototypeOf(error) !== typePrototype) throw 'protocol TypeError called-Realm';
        caught = true;
      }
      if (!caught || output !== previous || returnGets !== 0) throw 'protocol error without publication or close';
      const expectedNextGets = fault === 'next-noncallable' || fault === 'next-result' ? 1 : 0;
      if (nextGets !== expectedNextGets) throw 'protocol next Get precedence';
      nativeCases++;
    }
  }
  let crossReturnGets = 0;
  const crossIterator = {next() { return 1; }, get return() { crossReturnGets++; throw marker; }};
  let crossCaught = false;
  try { Reflect.construct(Constructor, [{[Symbol.iterator]() { return crossIterator; }}], OtherNewTarget); }
  catch (error) {
    if (Object.getPrototypeOf(error) !== typePrototype) throw 'protocol error does not borrow NewTarget Realm';
    crossCaught = true;
  }
  if (!crossCaught || crossReturnGets !== 0) throw 'cross-NewTarget protocol failure without close';
  nativeCases++;
}
if (nativeCases !== 22 || markerCases !== 30) throw 'both Realm call/Construct abrupt cohorts';
print('aggregate-error-iterator-abrupt:ok');
262;
