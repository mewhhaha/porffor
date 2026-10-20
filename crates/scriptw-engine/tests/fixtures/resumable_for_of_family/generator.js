function check(value) { if (!value) throw new Error('ordinary iterator phases'); }
const events = [], readers = [], whole = {marker: 41}; whole.self = whole;
let reads = 0, calls = 0;
const inner = {[Symbol.iterator]() { return {
  next() { return {done: false, value: undefined}; },
  return() { events.push('inner-close'); return {}; }
}; }};
const iterator = {
  get next() { reads++; return function () {
    check(this === iterator); calls++; return {done: false, value: inner};
  }; },
  return() { events.push('outer-close'); return {get then() { throw 'sync close adopted'; }}; }
};
const iterable = {[Symbol.iterator]() { events.push('acquire'); return iterator; }};
let headReader;
function* values() {
  try {
    for (const [item = yield 'default'] of (headReader = () => item, yield 'head')) {
      const retained = item; readers.push(() => retained); yield retained;
    }
  } finally { events.push('finally'); yield 'cleanup'; }
}
const stream = values();
check(stream.next().value === 'head' && events.length === 0);
try { headReader(); throw 'missing head TDZ'; } catch (error) { check(error instanceof ReferenceError); }
gc();
check(stream.next(iterable).value === 'default' && reads === 1 && calls === 1);
Object.defineProperty(iterator, 'next', {value() { throw 'method reloaded'; }});
gc();
const promise = Promise.resolve(7);
const received = stream.next(promise);
check(!received.done && received.value === promise && readers[0]() === promise);
check(events.join(',') === 'acquire,inner-close');
const returning = stream.return(whole);
check(!returning.done && returning.value === 'cleanup');
check(events.join(',') === 'acquire,inner-close,outer-close,finally');
gc();
const completed = stream.next(); check(completed.done && completed.value === whole);
check(readers[0]() === promise && reads === 1 && calls === 1);
try { headReader(); throw 'head TDZ initialized by body'; } catch (error) { check(error instanceof ReferenceError); }

const target = {x: 0}, key = {toString() { events.push('key'); return 'x'; }};
function* assign() { for (target[yield 'key'] of [8]) { yield target.x; break; } }
const assigning = assign(); check(assigning.next().value === 'key');
check(assigning.next(key).value === 8 && target.x === 8);
check(assigning.next().done && events[events.length - 1] === 'key');
print('ok');
