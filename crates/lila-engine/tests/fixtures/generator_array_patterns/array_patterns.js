function check(condition, label) { if (!condition) throw label; }
function next(iterator, input, expected, done) {
  var result = iterator.next(input);
  check(result.value === expected && result.done === done, 'unexpected-array-pattern-step');
  return result;
}
var whole = { marker: 7 }; whole.self = whole;
whole[Symbol.toPrimitive] = function () { throw 'unexpected-pattern-value-coercion'; };
function stream(values, events, label) {
  var source = { closes: 0 };
  source[Symbol.iterator] = function () {
    events.push(label + 'acquire');
    var position = 0;
    var record = { return: function () {
      events.push(label + 'close'); source.closes++;
      if (source.closeError !== undefined) throw source.closeError;
      return {};
    } };
    Object.defineProperty(record, 'next', { configurable: true, get: function () {
      events.push(label + 'next-get');
      return function () {
        var index = position++;
        events.push(label + 'step' + index);
        return {
          get done() { events.push(label + 'done' + index); return index >= values.length; },
          get value() { events.push(label + 'value' + index); return values[index]; }
        };
      };
    } });
    source.record = record;
    return record;
  };
  return source;
}

// Elision reads done without value. Member selection precedes the next step;
// its raw key converts only after the complete default has resumed.
var events = [], destination = {}, selected = Symbol('selected');
var rawKey = { [Symbol.toPrimitive]: function () { events.push('key'); return selected; } };
var source = stream([11, undefined, whole], events, '');
function* ordered(input) {
  return ([, (yield 'base')[yield 'raw'] = (yield 'first', yield 'second'), ...destination.rest] = input);
}
var iterator = ordered(source);
next(iterator, undefined, 'base', false);
check(events.join(',') === 'acquire,next-get,step0,done0', 'elision-never-reads-value');
gc(); next(iterator, destination, 'raw', false);
next(iterator, rawKey, 'first', false);
check(events.join(',') === 'acquire,next-get,step0,done0,step1,done1,value1', 'target-before-next-default');
Object.defineProperty(source.record, 'next', { value: function () { throw 'reloaded-next'; } });
gc(); next(iterator, whole, 'second', false);
next(iterator, whole, source, true);
check(events.join(',') === 'acquire,next-get,step0,done0,step1,done1,value1,key,step2,done2,value2,step3,done3', 'cached-next-raw-key-rest-order');
check(destination[selected] === whole && destination.rest.length === 1 && destination.rest[0] === whole, 'whole-value-and-rest');
check(source.closes === 0, 'exhausted-rest-skips-close');

// A nonundefined value skips all default states; a truncated live iterator
// closes exactly once, using the original record retained over the yield.
events = []; source = stream([whole, 2, 3], events, '');
function* lazy(input) { let [first = (yield 'wrong1', yield 'wrong2'), second = yield 'wrong3'] = input; yield first; return second; }
iterator = lazy(source); next(iterator, undefined, whole, false);
check(source.closes === 1, 'normal-truncation-close-before-later-yield');
gc(); next(iterator, undefined, 2, true);
events = []; source = stream([], events, '');
function* exhausted(input) { let [first = yield 'exhausted-first', second = yield 'exhausted-second'] = input; return [first, second]; }
iterator = exhausted(source); next(iterator, undefined, 'exhausted-first', false);
gc(); next(iterator, whole, 'exhausted-second', false);
var result = iterator.next(17);
check(result.done && result.value[0] === whole && result.value[1] === 17, 'exhausted-defaults-still-resume');
check(events.join(',') === 'acquire,next-get,step0,done0' && source.closes === 0, 'done-prevents-extra-next-and-close');

// Suspended var patterns expose initialized bindings before iteration starts,
// then keep the same cells through default, nested and rest writes.
function* hoistedVarArray(input) {
  var read = () => [first, nested, rest];
  yield read;
  var [first = yield 'new-array-var', {nested}, ...rest] = input;
  return read;
}
iterator = hoistedVarArray([undefined, {nested: whole}, whole]);
var readArrayVars = iterator.next().value;
var beforeArrayVars = readArrayVars();
check(beforeArrayVars[0] === undefined && beforeArrayVars[1] === undefined && beforeArrayVars[2] === undefined, 'array-var-names-initialized-before-pattern');
next(iterator, undefined, 'new-array-var', false);
gc(); result = iterator.next(whole);
var afterArrayVars = readArrayVars();
check(result.done && result.value === readArrayVars && afterArrayVars[0] === whole && afterArrayVars[1] === whole && afterArrayVars[2][0] === whole, 'array-var-captures-retain-resumed-writes');
function* skippedArrayVarDefault() { var [value = yield 'wrong-array-var-default'] = [whole]; return value; }
next(skippedArrayVarDefault(), undefined, whole, true);

// Protocol failures mark the actual record done. Neither the original Throw
// nor a malformed iterator result is followed by IteratorClose.
function failedProtocol(kind, token) {
  var closes = 0;
  var input = { [Symbol.iterator]: function () { return {
    next: function () {
      if (kind === 'next') throw token;
      if (kind === 'result') return 7;
      return {
        get done() { if (kind === 'done') throw token; return false; },
        get value() { throw token; }
      };
    },
    return: function () { closes++; return {}; }
  }; } };
  function* consume() { var value; [value = yield 'unreached'] = input; }
  try { consume().next(); throw 'missing-protocol-error'; }
  catch (error) { check(kind === 'result' ? error instanceof TypeError : error === token, 'protocol-whole-error'); }
  check(closes === 0, 'protocol-failure-does-not-close');
}
for (var failureKind of ['next', 'result', 'done', 'value']) failedProtocol(failureKind, whole);

// Target/default failures occur inside the close obligation. Incoming Throw
// wins a throwing return method; incoming Return can be replaced by its Throw.
events = []; source = stream([undefined], events, ''); source.closeError = whole;
function targetFailure() { throw selected; }
function* beforeStep(input) { [targetFailure()[yield 'unreached-target']] = input; }
try { beforeStep(source).next(); throw 'missing-target-throw'; }
catch (error) { check(error === selected, 'incoming-target-throw-wins-close-throw'); }
check(events.join(',') === 'acquire,next-get,close' && source.closes === 1, 'target-failure-closes-before-step');
function failDefault() { throw selected; }
function* defaultFailure(input) { var value; [value = (yield 'default-before-throw', failDefault())] = input; }
events = []; source = stream([undefined], events, ''); source.closeError = whole;
iterator = defaultFailure(source); next(iterator, undefined, 'default-before-throw', false);
try { iterator.next(); throw 'missing-default-throw'; }
catch (error) { check(error === selected && source.closes === 1, 'default-throw-close-precedence'); }
destination.completed = 0;
function* normalClose(input) { [destination.completed = yield 'normal-default'] = input; }
events = []; source = stream([undefined], events, ''); source.closeError = whole;
iterator = normalClose(source); next(iterator, undefined, 'normal-default', false);
try { iterator.next(23); throw 'missing-normal-close-throw'; }
catch (error) { check(error === whole && destination.completed === 23 && source.closes === 1, 'normal-close-throw-after-put'); }

destination.pending = 5;
function* interrupted(input) {
  try { [destination.pending = yield 'pending-default'] = input; }
  finally { yield 'pending-finally'; }
}
events = []; source = stream([undefined], events, ''); iterator = interrupted(source);
next(iterator, undefined, 'pending-default', false);
result = iterator.return(whole);
check(result.value === 'pending-finally' && !result.done && source.closes === 1, 'return-closes-before-yielding-finally');
gc(); next(iterator, undefined, whole, true); check(destination.pending === 5, 'return-skips-put');
events = []; source = stream([undefined], events, ''); source.closeError = selected;
iterator = interrupted(source); next(iterator, undefined, 'pending-default', false);
result = iterator.return(whole);
check(result.value === 'pending-finally' && !result.done && source.closes === 1, 'close-throw-before-return-finally');
try { iterator.next(); throw 'missing-return-close-error'; }
catch (error) { check(error === selected, 'close-throw-replaces-return'); }
events = []; source = stream([undefined], events, ''); source.closeError = selected;
iterator = interrupted(source); next(iterator, undefined, 'pending-default', false);
result = iterator.throw(whole); check(result.value === 'pending-finally' && !result.done && source.closes === 1, 'throw-closes-before-finally');
gc(); try { iterator.next(); throw 'missing-injected-throw'; }
catch (error) { check(error === whole && destination.pending === 5, 'incoming-throw-preserved'); }

function* caught(firstInput, secondInput) {
  var value = 0;
  try { [value = yield 'caught-default'] = firstInput; }
  catch (error) { yield error; }
  [value = yield 'fresh-default'] = secondInput;
  return value;
}
events = []; var oldInput = stream([undefined], events, 'old-'), freshInput = stream([undefined], events, 'fresh-');
iterator = caught(oldInput, freshInput); next(iterator, undefined, 'caught-default', false);
result = iterator.throw(whole);
check(result.value === whole && !result.done && oldInput.closes === 1 && freshInput.closes === 0, 'close-before-yielding-catch');
gc(); next(iterator, undefined, 'fresh-default', false); next(iterator, 31, 31, true);
check(oldInput.closes === 1 && freshInput.closes === 1, 'fresh-pattern-after-caught-abrupt');

// Nested active records close from inner to outer, even when the completion
// is injected while the inner default owns several suspension states.
events = [];
var inner = stream([undefined], events, 'inner-');
var outer = stream([inner], events, 'outer-');
function* nested(input) { var value; try { [[value = (yield 'nested-first', yield 'nested-second')]] = input; } finally { yield 'nested-finally'; } }
iterator = nested(outer); next(iterator, undefined, 'nested-first', false);
gc(); next(iterator, whole, 'nested-second', false);
result = iterator.throw(whole);
check(result.value === 'nested-finally' && !result.done, 'nested-yielding-finally');
check(events.slice(-2).join(',') === 'inner-close,outer-close' && inner.closes === 1 && outer.closes === 1, 'nested-close-order');
try { iterator.next(); throw 'missing-nested-whole-throw'; }
catch (error) { check(error === whole, 'nested-whole-throw'); }

// Rest target evaluation can suspend before draining. A nested rest pattern
// then uses its own iterator, rather than replaying the outer source.
events = []; source = stream([undefined, whole], events, '');
function* restTarget(input) { return ([...(yield 'rest-base')[yield 'rest-key']] = input); }
iterator = restTarget(source); next(iterator, undefined, 'rest-base', false);
next(iterator, destination, 'rest-key', false);
check(events.join(',') === 'acquire,next-get', 'rest-target-before-drain');
next(iterator, 'tail', source, true);
check(destination.tail.length === 2 && destination.tail[1] === whole && source.closes === 0, 'rest-whole-values');
events = []; source = stream([undefined, whole], events, '');
function* restPattern(input) { let [...[first = yield 'rest-default', second]] = input; return [first, second]; }
iterator = restPattern(source); next(iterator, undefined, 'rest-default', false);
gc(); result = iterator.next(9);
check(result.done && result.value[0] === 9 && result.value[1] === whole && source.closes === 0, 'nested-rest-pattern-after-drain');

// Acquisition is after the complete RHS. Abandoning the RHS never opens an
// iterator close obligation for a value that has not been acquired.
events = []; source = stream([undefined], events, '');
function* rhs() { var value; return ([value = yield 'rhs-default'] = yield 'rhs'); }
iterator = rhs(); next(iterator, undefined, 'rhs', false);
next(iterator, source, 'rhs-default', false); gc(); next(iterator, whole, source, true);
check(source.closes === 1, 'rhs-before-acquisition');
events = [];
iterator = rhs(); next(iterator, undefined, 'rhs', false); result = iterator.return(whole);
check(result.done && result.value === whole && events.length === 0, 'return-in-rhs-has-no-iterator');

function* tdz(input) { let [first = (yield 'tdz', later), later] = input; }
events = []; source = stream([undefined, 8], events, ''); iterator = tdz(source);
next(iterator, undefined, 'tdz', false);
try { iterator.next(); throw 'missing-array-tdz'; }
catch (error) { check(error instanceof ReferenceError && source.closes === 1, 'later-lexical-target-tdz'); }
function* head(input) { for (let [i = yield 'head-default', step] = input; i < 2; i++) yield () => [i, step]; }
iterator = head([undefined, 3]); next(iterator, undefined, 'head-default', false);
var firstClosure = iterator.next(0).value; gc(); var secondClosure = iterator.next().value;
check(firstClosure()[0] === 0 && secondClosure()[0] === 1 && firstClosure()[1] === 3 && secondClosure()[1] === 3, 'array-head-distinct-iteration-cells');
check(iterator.next().done, 'array-head-completes');

var converted = 0;
var nullKey = { [Symbol.toPrimitive]: function () { converted++; return 'value'; } };
function* nullishTarget(input) { [(yield 'null-base')[yield 'null-key'] = yield 'null-default'] = input; }
events = []; source = stream([undefined], events, ''); iterator = nullishTarget(source);
next(iterator, undefined, 'null-base', false); next(iterator, null, 'null-key', false);
next(iterator, nullKey, 'null-default', false);
try { iterator.next(whole); throw 'missing-array-nullish-error'; }
catch (error) { check(error instanceof TypeError && converted === 0 && source.closes === 1, 'nullish-put-before-key-coercion-closes'); }

class Holder {
  #value;
  *fill(input) { return ([(yield 'private-base').#value = yield 'private-default'] = input); }
  read() { return this.#value; }
}
var holder = new Holder(); events = []; source = stream([undefined], events, '');
iterator = holder.fill(source); next(iterator, undefined, 'private-base', false);
next(iterator, holder, 'private-default', false); gc(); next(iterator, whole, source, true);
check(holder.read() === whole && source.closes === 1, 'retained-private-target');
events = []; source = stream([undefined], events, ''); iterator = holder.fill(source);
next(iterator, undefined, 'private-base', false); next(iterator, {}, 'private-default', false);
try { iterator.next(whole); throw 'missing-array-brand-error'; }
catch (error) { check(error instanceof TypeError && source.closes === 1, 'brand-error-after-default-closes'); }

// Delegated done:false also keeps the same outer IteratorRecord alive.
var delegateIndex = 0;
var delegated = { [Symbol.iterator]: function () { return { next: function () {
  delegateIndex++; return delegateIndex === 1 ? {value: 'delegate', done: false} : {value: whole, done: true};
} }; } };
events = []; source = stream([undefined], events, '');
function* delegateDefault(input) { let [value = yield* delegated] = input; return value; }
iterator = delegateDefault(source); next(iterator, undefined, 'delegate', false);
check(source.closes === 0, 'delegate-done-false-keeps-close-obligation');
gc(); next(iterator, undefined, whole, true); check(source.closes === 1, 'delegate-completion-closes-once');
print('generator-array-patterns:ok');
