function same(actual, expected, label) {
  if (actual !== expected) throw label + ': ' + actual + ' !== ' + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + ' value');
  same(result.done, done, label + ' done');
}
const state = { next: 0, nextGets: 0, closes: 0, keys: 0, defaults: 0, gets: 0 };
const log = [];
const rows = [
  { get item() { ++state.gets; log.push('get:1'); return 10; }, other: 101 },
  { get item() { ++state.gets; log.push('get:2'); return undefined; }, other: 202 }
];
function cachedNext() {
  same(this, iterator, 'cached next receiver');
  same(arguments.length, 0, 'cached next arguments');
  const index = state.next++;
  log.push('next:' + index);
  return index < rows.length ? { value: rows[index], done: false } : { done: true };
}
let nextMethod = cachedNext;
const iterator = {
  get next() { ++state.nextGets; log.push('get:next'); return nextMethod; },
  set next(value) { nextMethod = value; },
  return() { ++state.closes; return {}; }
};
const source = { [Symbol.iterator]() { log.push('open'); return iterator; } };
function key() {
  ++state.keys;
  log.push('key');
  iterator.next = function () { throw 'replacement next must remain uncalled'; };
  return 'item';
}
function fallback() { ++state.defaults; log.push('default'); return 20; }
function* walk(iterable) {
  for (let { [key()]: value = fallback(), ...rest } of iterable) {
    let body = value * 10;
    const readers = [() => value, () => body, () => rest.other];
    log.push('body:' + value);
    yield readers;
    ++value;
    ++body;
    log.push('resume:' + value);
    yield readers;
  }
  return 'done';
}
const consumer = walk(source);
const first = consumer.next().value;
same(first[0](), 10, 'first head cell');
same(first[1](), 100, 'first body cell');
same(first[2](), 101, 'first rest cell');
same(log.join(','), 'open,get:next,next:0,key,get:1,body:10', 'one Get before default decision');
same(state.gets, 1, 'nonundefined default path Get once');
same(state.defaults, 0, 'nonundefined default skipped');
step(consumer.next(), first, false, 'first resume');
same(first[0](), 11, 'mutable head resumes with same cell');
same(first[1](), 101, 'body resumes with same cell');
same(state.keys, 1, 'resume does not repeat computed key');
same(state.gets, 1, 'resume does not repeat destructuring Get');
same(state.next, 1, 'resume does not advance outer iterator');
const second = consumer.next().value;
same(second[0](), 20, 'second default value');
same(second[1](), 200, 'fresh second body cell');
same(second[2](), 202, 'fresh second rest cell');
same(first[0](), 11, 'earlier head cell stays captured');
same(state.defaults, 1, 'default evaluated once at second entry');
same(state.gets, 2, 'each source property observed once');
same(state.nextGets, 1, 'outer next cached across head hooks');
step(consumer.next(), second, false, 'second resume');
same(second[0](), 21, 'second head update retained');
step(consumer.next(), 'done', true, 'normal exhaustion');
same(state.closes, 0, 'normal exhaustion does not close');
same(state.keys, 2, 'only actual entries initialize');
same(state.next, 3, 'two values and terminal done');
same(log.join(','), 'open,get:next,next:0,key,get:1,body:10,resume:11,next:1,key,get:2,default,body:20,resume:21,next:2', 'full head and resume chronology');
step(consumer.next(), undefined, true, 'completed consumer');
same(state.gets, 2, 'completed consumer has no head effects');

function* interleave(iterable) {
  for (const [first, { nested }, ...rest] of iterable) {
    const readers = [() => first, () => nested, () => rest.join(',')];
    yield readers;
    yield readers;
  }
  return 'done';
}
const a = interleave([[1, { nested: 2 }, 3, 4], [5, { nested: 6 }, 7]]);
const b = interleave([[11, { nested: 12 }, 13], [15, { nested: 16 }, 17, 18]]);
const aFirst = a.next().value;
const bFirst = b.next().value;
same(aFirst[0](), 1, 'first instance head');
same(aFirst[1](), 2, 'first nested binding');
same(aFirst[2](), '3,4', 'first rest binding');
same(bFirst[0](), 11, 'second instance head');
step(a.next(), aFirst, false, 'first instance resume');
step(b.next(), bFirst, false, 'second instance resume');
const aSecond = a.next().value;
const bSecond = b.next().value;
same(aSecond[0](), 5, 'fresh next first-instance head');
same(bSecond[1](), 16, 'fresh next second-instance nested binding');
same(bSecond[2](), '17,18', 'second-instance rest');
same(aFirst[0](), 1, 'old first cell survives next entry');
same(bFirst[1](), 12, 'old second nested cell survives next entry');
step(a.next(), aSecond, false, 'later first instance resume');
step(b.next(), bSecond, false, 'later second instance resume');
step(a.next(), 'done', true, 'first instance exhaustion');
step(b.next(), 'done', true, 'second instance exhaustion');

function* defaults(iterable) {
  for (let [first = 3, second = first + 4] of iterable) { yield [first, second]; }
}
const defaultConsumer = defaults([[undefined, undefined]]);
const defaultValues = defaultConsumer.next().value;
same(defaultValues.join(','), '3,7', 'earlier initialized binding is available to later default');
step(defaultConsumer.next(), undefined, true, 'default consumer completes');
function* strings(iterable) { for (const [first, ...rest] of iterable) { yield first + ':' + rest.join(''); } }
const stringConsumer = strings(['abc']);
step(stringConsumer.next(), 'a:bc', false, 'array binding iterates primitive String');
step(stringConsumer.next(), undefined, true, 'primitive String binding completes');
print('ok');
262;
