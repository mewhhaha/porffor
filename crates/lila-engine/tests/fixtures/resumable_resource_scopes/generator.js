function check(value, message) { if (!value) throw new Error(message); }
const events = [];
function resource(name, failure) {
  const value = {name};
  Object.defineProperty(value, Symbol.dispose, {configurable: true, get() {
    events.push('get:' + name);
    return function () { check(this === value, 'disposal receiver'); events.push('dispose:' + name); if (failure) throw failure; };
  }});
  return value;
}
const whole = {marker: 1}; whole.self = whole;
let readSecond;
function* registrations() {
  try {
    readSecond = () => second;
    using first = yield 'first', second = yield 'second';
    yield first.name + second.name;
  } finally { gc(); yield 'finally'; }
  return whole;
}
const first = resource('a');
const stream = registrations();
check(stream.next().value === 'first', 'first initializer');
check(events.length === 0, 'nothing registered before acquisition');
check(stream.next(first).value === 'second', 'second initializer');
check(events.join(',') === 'get:a', 'first acquisition once');
let tdz = false; try { readSecond(); } catch (error) { tdz = error instanceof ReferenceError; }
check(tdz, 'second binding stays uninitialized');
gc();
Object.defineProperty(first, Symbol.dispose, {get() { throw new Error('method replay'); }});
check(stream.return(whole).value === 'finally', 'injected Return reaches finalizer after disposal');
check(events.join(',') === 'get:a,dispose:a', 'only registered entries close');
const returned = stream.next(); check(returned.done && returned.value === whole, 'whole Return identity');
events.length = 0;
function* head() {
  const readers = []; let count = 0;
  for (using held = yield 'acquire'; (yield 'test', count < 2); (yield 'update', ++count)) {
    readers.push(() => held); yield count;
    if (count === 0) continue;
  }
  check(readers[0]() === forValue && readers[1]() === forValue, 'one head binding through back edges');
  return whole;
}
const forValue = resource('head');
const loop = head(); check(loop.next().value === 'acquire', 'head initializer');
check(loop.next(forValue).value === 'test', 'test after registration');
check(loop.next().value === 0, 'first body'); gc();
check(loop.next().value === 'update', 'Continue reaches update');
check(events.join(',') === 'get:head', 'Continue retains whole head capability');
check(loop.next().value === 'test', 'back edge');
check(loop.next().value === 1, 'second body');
check(loop.next().value === 'update', 'normal update');
check(loop.next().value === 'test', 'false test phase');
const ended = loop.next();
check(ended.done && ended.value === whole, 'loop return');
check(events.join(',') === 'get:head,dispose:head', 'head closes once at false test');
events.length = 0;
function* caseValues() {
  outer: switch (yield 'select') {
    case (yield 'case'): {
      using a = resource('case-a'); yield 'fallthrough';
    }
    default: {
      using b = resource('case-b'); gc(); yield () => b; break outer;
    }
  }
  return whole;
}
const caseStream = caseValues(); check(caseStream.next().value === 'select', 'discriminant');
check(caseStream.next(0).value === 'case', 'lazy selector');
check(caseStream.next(0).value === 'fallthrough', 'matched case');
const reader = caseStream.next().value; check(reader().name === 'case-b', 'captured block binding');
check(events.join(',') === 'get:case-a,dispose:case-a,get:case-b', 'fallthrough closes the first block before entering the second');
gc(); check(caseStream.next().value === whole, 'labelled break completion');
check(events.join(',') === 'get:case-a,dispose:case-a,get:case-b,dispose:case-b', 'labelled break closes the active block once');
print('resumable-generator-resource-scopes:ok');
