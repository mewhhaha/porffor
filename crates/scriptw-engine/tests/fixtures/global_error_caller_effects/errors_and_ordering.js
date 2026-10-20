function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'replaced constructor or unreachable raw operand'; }
const define = Object.defineProperty;
const getPrototypeOf = Object.getPrototypeOf;
const own = Object.prototype.hasOwnProperty;
const savedError = Error;
const savedAggregate = AggregateError;
const savedToString = Error.prototype.toString;

// Literal native calls exercise each constructor's own synchronous effect metadata.
function errorEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const result = Error({toString() {
    kind = function() { return 7; }; shape = {value: Symbol.for('Error effect')};
    elements[0] = function() { return 9; }; return 'Error message';
  }});
  check(result.message === 'Error message' && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'Error message hook invalidates captured facts');
}
function evalEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const result = EvalError({toString() {
    kind = function() { return 7; }; shape = {value: Symbol.for('EvalError effect')};
    elements[0] = function() { return 9; }; return 'Eval message';
  }});
  check(result.message === 'Eval message' && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'EvalError message hook invalidates captured facts');
}
function rangeEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const result = RangeError({toString() {
    kind = function() { return 7; }; shape = {value: Symbol.for('RangeError effect')};
    elements[0] = function() { return 9; }; return 'Range message';
  }});
  check(result.message === 'Range message' && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'RangeError message hook invalidates captured facts');
}
function referenceEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const result = ReferenceError({toString() {
    kind = function() { return 7; }; shape = {value: Symbol.for('ReferenceError effect')};
    elements[0] = function() { return 9; }; return 'Reference message';
  }});
  check(result.message === 'Reference message' && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'ReferenceError message hook invalidates captured facts');
}
function syntaxEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const result = SyntaxError({toString() {
    kind = function() { return 7; }; shape = {value: Symbol.for('SyntaxError effect')};
    elements[0] = function() { return 9; }; return 'Syntax message';
  }});
  check(result.message === 'Syntax message' && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'SyntaxError message hook invalidates captured facts');
}
function typeEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const result = TypeError({toString() {
    kind = function() { return 7; }; shape = {value: Symbol.for('TypeError effect')};
    elements[0] = function() { return 9; }; return 'Type message';
  }});
  check(result.message === 'Type message' && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'TypeError message hook invalidates captured facts');
}
function uriEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const result = URIError({toString() {
    kind = function() { return 7; }; shape = {value: Symbol.for('URIError effect')};
    elements[0] = function() { return 9; }; return 'URI message';
  }});
  check(result.message === 'URI message' && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'URIError message hook invalidates captured facts');
}
errorEffects(); evalEffects(); rangeEffects(); referenceEffects();
syntaxEffects(); typeEffects(); uriEffects();

function causeEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const trace = [], cause = {};
  const options = new Proxy({}, {
    has(target, key) {
      check(key === 'cause', 'InstallErrorCause uses HasProperty'); trace.push('has');
      kind = function() { return 7; }; return true;
    },
    get(target, key, rawThis) {
      check(key === 'cause' && rawThis === options, 'InstallErrorCause retains options receiver');
      trace.push('get'); shape = {value: Symbol.for('cause effect')};
      elements[0] = function() { return 9; }; return cause;
    }
  });
  const result = Error('cause message', options);
  check(result.cause === cause && trace.join(',') === 'has,get' && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'cause Has/Get hooks invalidate caller facts');
}
causeEffects();

function newTargetEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const trace = [], prototype = {}, cause = {};
  const message = {toString() { trace.push('message'); shape = {value: Symbol.for('prefix effect')}; return 'prefix'; }};
  const options = new Proxy({}, {
    has(target, key) { check(key === 'cause', 'prefix cause Has'); trace.push('has'); return true; },
    get(target, key) { check(key === 'cause', 'prefix cause Get'); trace.push('get'); elements[0] = function() { return 9; }; return cause; }
  });
  const newTarget = new Proxy(function() { throw 'NewTarget body is not called'; }, {get(target, key, rawThis) {
    if (key === 'prototype') { trace.push('prototype'); kind = function() { return 7; }; return prototype; }
    return Reflect.get(target, key, rawThis);
  }});
  const result = Reflect.construct(Error, [message, options], newTarget);
  check(getPrototypeOf(result) === prototype && result.message === 'prefix' && result.cause === cause &&
    trace.join(',') === 'prototype,message,has,get' && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'NewTarget prototype precedes message and cause');
}
newTargetEffects();

function aggregateEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const trace = [], first = {}, cause = {};
  let step = 0, closes = 0;
  const iterator = {get next() {
    trace.push('next.get');
    return function() {
      check(this === iterator && arguments.length === 0, 'Aggregate cached next receiver and argc');
      trace.push('next' + step);
      if (step++ === 0) return {get done() { trace.push('done'); return false; },
        get value() { trace.push('value'); elements[0] = function() { return 9; }; return first; }};
      return {get done() { trace.push('terminal.done'); return true; },
        get value() { throw 'terminal Aggregate value'; }};
    };
  }, get return() { closes++; throw 'Aggregate list does not close'; }};
  const errors = {get [Symbol.iterator]() {
    trace.push('iterator.get'); shape = {value: Symbol.for('Aggregate effect')};
    return function() { check(this === errors, 'Aggregate iterator receiver'); trace.push('iterator.call'); return iterator; };
  }};
  const message = {toString() { trace.push('message'); kind = function() { return 7; }; return 'aggregate'; }};
  const options = new Proxy({}, {
    has(target, key) { check(key === 'cause', 'Aggregate cause Has'); trace.push('has'); return true; },
    get(target, key) { check(key === 'cause', 'Aggregate cause Get'); trace.push('get'); return cause; }
  });
  const result = AggregateError(errors, message, options);
  check(result.message === 'aggregate' && result.cause === cause && result.errors.length === 1 && result.errors[0] === first &&
    trace.join(',') === 'message,has,get,iterator.get,iterator.call,next.get,next0,done,value,next1,terminal.done' &&
    closes === 0 && typeof kind === 'function' && kind() === 7 && typeof shape.value === 'symbol' &&
    typeof elements[0] === 'function' && elements[0]() === 9, 'Aggregate prefix and iteration caller effects');
}
aggregateEffects();

function suppressedEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const error = {toString: poison}, suppressed = {toString: poison};
  const unusedOptions = new Proxy({}, {has: poison, get: poison});
  const result = SuppressedError(error, suppressed, {toString() {
    kind = function() { return 7; }; shape = {value: Symbol.for('Suppressed effect')};
    elements[0] = function() { return 9; }; return 'suppressed message';
  }}, unusedOptions);
  check(result.error === error && result.suppressed === suppressed && result.message === 'suppressed message' &&
    !own.call(result, 'cause') && typeof kind === 'function' && kind() === 7 &&
    typeof shape.value === 'symbol' && typeof elements[0] === 'function' && elements[0]() === 9,
    'SuppressedError coerces only third message and has no cause options');
}
suppressedEffects();

function toStringEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const trace = [], receiver = {};
  define(receiver, 'name', {configurable: true, get() {
    trace.push('name.get'); kind = function() { return 7; };
    return {toString() { trace.push('name.string'); shape = {value: Symbol.for('name effect')}; return 'Prepared'; }};
  }});
  define(receiver, 'message', {get() {
    trace.push('message.get'); define(receiver, 'name', {value: 'replacement', configurable: true});
    return {toString() { trace.push('message.string'); elements[0] = function() { return 9; }; return 'Message'; }};
  }});
  receiver.saved = Error.prototype.toString; receiver.toString = poison;
  const result = receiver.saved();
  check(result === 'Prepared: Message' && trace.join(',') === 'name.get,name.string,message.get,message.string' &&
    typeof kind === 'function' && kind() === 7 && typeof shape.value === 'symbol' &&
    typeof elements[0] === 'function' && elements[0]() === 9,
    'Error.toString retains prepared name and invalidates Get/ToString caller facts');
}
toStringEffects();

const argumentTrace = [];
const receiver = {Error: poison};
receiver.saved = new Proxy(savedError, {apply(target, rawThis, args) {
  argumentTrace.push('apply');
  check(rawThis === receiver && args.length === 4 && args[1] === undefined && args[2] === 13 && args[3] === 17,
    'Error alias retains raw receiver and complete ignored operands');
  return Reflect.apply(target, rawThis, args);
}});
function messageArgument() {
  argumentTrace.push('first'); receiver.saved = poison;
  return {toString() { argumentTrace.push('coerce'); return 'retained'; }};
}
const acquired = receiver.saved(messageArgument(), undefined, (argumentTrace.push('extra1'), 13),
  (argumentTrace.push('extra2'), 17));
check(acquired.message === 'retained' && argumentTrace.join(',') === 'first,extra1,extra2,apply,coerce',
  'Error alias is acquired before argument replacement and native coercion');

const spreadTrace = [];
const spreadReceiver = {AggregateError: poison, saved: savedAggregate};
let spreadPosition = 0, spreadCloses = 0;
const spreadMessage = {toString() { spreadTrace.push('message'); return 'spread aggregate'; }};
const spreadIterator = {next() {
  spreadTrace.push('next' + spreadPosition);
  if (spreadPosition++ === 0) return {done: false, value: [3, 5]};
  if (spreadPosition === 2) return {done: false, value: spreadMessage};
  if (spreadPosition === 3) return {done: false, value: undefined};
  return {done: true};
}, get return() { spreadCloses++; throw 'argument spread does not close'; }};
const spread = {[Symbol.iterator]() { spreadTrace.push('iterator'); spreadReceiver.saved = poison; return spreadIterator; }};
const spreadResult = spreadReceiver.saved(...spread, (spreadTrace.push('extra'), 19));
check(spreadResult.message === 'spread aggregate' && spreadResult.errors.join(',') === '3,5' && spreadCloses === 0 &&
  spreadTrace.join(',') === 'iterator,next0,next1,next2,next3,extra,message',
  'Aggregate acquired alias and real spread finish all operands before message/list phases');
check(savedToString.call({name: '', message: 'only'}) === 'only' &&
  savedToString.call({name: 'Only', message: ''}) === 'Only', 'Error.toString retained empty-field rules');
print('global-error-errors:ok');
262;
