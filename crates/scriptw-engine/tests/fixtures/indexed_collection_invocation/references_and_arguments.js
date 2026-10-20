function check(condition, name) { if (!condition) throw name; }
function elements(actual, expected) {
  check(actual.length === expected.length, 'length');
  for (let index = 0; index < expected.length; ++index) {
    check(actual[index] === expected[index], 'element');
  }
}
function poison() { throw 'canonical key must not be read'; }

// Literal intrinsic assignments keep each target separate during inference.
const push = {length: 1, 0: 4, alias: Array.prototype.push, push: poison};
check(push.alias(5) === 2 && push[1] === 5, 'push alias');
const pop = {length: 2, 0: 4, 1: 5, alias: Array.prototype.pop, pop: poison};
check(pop.alias() === 5 && pop.length === 1, 'pop alias');
const shift = {length: 2, 0: 4, 1: 5, alias: Array.prototype.shift, shift: poison};
check(shift.alias() === 4 && shift.length === 1 && shift[0] === 5, 'shift alias');
const unshift = {length: 1, 0: 5, alias: Array.prototype.unshift, unshift: poison};
check(unshift.alias(4) === 2 && unshift[0] === 4 && unshift[1] === 5, 'unshift alias');
const fill = {length: 3, 0: 1, 1: 2, 2: 3, alias: Array.prototype.fill, fill: poison};
check(fill.alias(8, 1, 2) === fill && fill[0] === 1 && fill[1] === 8 && fill[2] === 3, 'fill alias');
const sort = {length: 3, 0: 3, 1: 1, 2: 2, alias: Array.prototype.sort, sort: poison};
check(sort.alias((a, b) => a - b) === sort, 'sort identity');
elements(sort, [1, 2, 3]);
const keys = {length: 2, alias: Array.prototype.keys, keys: poison};
const keyIterator = keys.alias();
check(keyIterator.next().value === 0 && keyIterator.next().value === 1 && keyIterator.next().done, 'keys alias');
const entries = {length: 1, 0: 'entry', alias: Array.prototype.entries, entries: poison};
const entryIterator = entries.alias();
elements(entryIterator.next().value, [0, 'entry']);
check(entryIterator.next().done, 'entries alias');
const values = {length: 1, 0: 'value', alias: Array.prototype.values, values: poison};
const valueIterator = values.alias();
check(valueIterator.next().value === 'value' && valueIterator.next().done, 'values alias');
const concat = {length: 1, 0: 4, alias: Array.prototype.concat, concat: poison};
const concatResult = concat.alias(5);
check(concatResult[0] === concat && concatResult[1] === 5 && concatResult.length === 2, 'concat generic alias');
const join = {length: 2, 0: 'a', 1: 'b', alias: Array.prototype.join, join: poison};
check(join.alias(':') === 'a:b', 'join alias');
const slice = {length: 3, 0: 4, 1: 5, 2: 6, alias: Array.prototype.slice, slice: poison};
elements(slice.alias(1), [5, 6]);
const splice = {length: 3, 0: 4, 1: 5, 2: 6, alias: Array.prototype.splice, splice: poison};
elements(splice.alias(1, 1, 9), [5]);
elements(splice, [4, 9, 6]);
const toString = {alias: Array.prototype.toString, toString: poison, join() { return 39; }};
check(toString.alias() === 39, 'shared toString alias');
const locale = {length: 0, alias: Array.prototype.toLocaleString, toLocaleString: poison};
check(locale.alias() === '', 'locale alias');
const flat = {length: 2, 0: [1, [2]], 1: 3, alias: Array.prototype.flat, flat: poison};
elements(flat.alias(2), [1, 2, 3]);
const flatMap = {length: 2, 0: 1, 1: 2, alias: Array.prototype.flatMap, flatMap: poison};
elements(flatMap.alias(value => [value, value + 1]), [1, 2, 2, 3]);
const at = {length: 2, 0: 4, 1: 5, alias: Array.prototype.at, at: poison};
check(at.alias(-1) === 5, 'at alias');
const reversed = {length: 2, 0: 4, 1: 5, alias: Array.prototype.toReversed, toReversed: poison};
elements(reversed.alias(), [5, 4]);
const spliced = {length: 2, 0: 4, 1: 5, alias: Array.prototype.toSpliced, toSpliced: poison};
elements(spliced.alias(1, 1, 9), [4, 9]);
const sorted = {length: 2, 0: 4, 1: 1, alias: Array.prototype.toSorted, toSorted: poison};
elements(sorted.alias(), [1, 4]);
const withValue = {length: 2, 0: 4, 1: 5, alias: Array.prototype.with, with: poison};
elements(withValue.alias(-1, 9), [4, 9]);
const reverse = {length: 2, 0: 4, 1: 5, alias: Array.prototype.reverse, reverse: poison};
check(reverse.alias() === reverse, 'reverse identity');
elements(reverse, [5, 4]);
const copy = {length: 3, 0: 1, 1: 2, 2: 3, alias: Array.prototype.copyWithin, copyWithin: poison};
check(copy.alias(1, 0, 1) === copy, 'copyWithin identity');
elements(copy, [1, 1, 3]);
const includes = {length: 2, 0: 4, 1: NaN, alias: Array.prototype.includes, includes: poison};
check(includes.alias(NaN), 'includes alias');
const index = {length: 3, 0: 4, 1: 5, 2: 4, alias: Array.prototype.indexOf, indexOf: poison};
check(index.alias(4, 1) === 2, 'indexOf alias');
const lastIndex = {length: 3, 0: 4, 1: 5, 2: 4, alias: Array.prototype.lastIndexOf, lastIndexOf: poison};
check(lastIndex.alias(4, 1) === 0, 'lastIndexOf alias');
const find = {length: 2, 0: 4, 1: 5, alias: Array.prototype.find, find: poison};
check(find.alias(value => value > 4) === 5, 'find alias');
const findIndex = {length: 2, 0: 4, 1: 5, alias: Array.prototype.findIndex, findIndex: poison};
check(findIndex.alias(value => value > 4) === 1, 'findIndex alias');
const findLast = {length: 3, 0: 4, 1: 5, 2: 6, alias: Array.prototype.findLast, findLast: poison};
check(findLast.alias(value => value > 4) === 6, 'findLast alias');
const findLastIndex = {length: 3, 0: 4, 1: 5, 2: 6, alias: Array.prototype.findLastIndex, findLastIndex: poison};
check(findLastIndex.alias(value => value > 4) === 2, 'findLastIndex alias');
const every = {length: 2, 0: 4, 1: 5, alias: Array.prototype.every, every: poison};
check(every.alias(value => value >= 4), 'every alias');
const some = {length: 2, 0: 4, 1: 5, alias: Array.prototype.some, some: poison};
check(some.alias(value => value > 4), 'some alias');
let eachTotal = 0;
const each = {length: 2, 0: 4, 1: 5, alias: Array.prototype.forEach, forEach: poison};
check(each.alias(value => { eachTotal += value; }) === undefined && eachTotal === 9, 'forEach alias');
const filter = {length: 2, 0: 4, 1: 5, alias: Array.prototype.filter, filter: poison};
elements(filter.alias(value => value > 4), [5]);
const map = {length: 2, 0: 4, 1: 5, alias: Array.prototype.map, map: poison};
elements(map.alias(value => value + 1), [5, 6]);
const reduce = {length: 2, 0: 4, 1: 5, alias: Array.prototype.reduce, reduce: poison};
check(reduce.alias((sum, value) => sum + value, 1) === 10, 'reduce alias');
const reduceRight = {length: 2, 0: 'a', 1: 'b', alias: Array.prototype.reduceRight, reduceRight: poison};
check(reduceRight.alias((text, value) => text + value, '') === 'ba', 'reduceRight alias');

const typed = new Int16Array([4, 5, 4]);
typed.includesAlias = Int16Array.prototype.includes; typed.includes = poison;
check(typed.includesAlias(5), 'TypedArray includes alias');
typed.indexAlias = Int16Array.prototype.indexOf; typed.indexOf = poison;
check(typed.indexAlias(4, 1) === 2, 'TypedArray indexOf alias');
typed.lastIndexAlias = Int16Array.prototype.lastIndexOf; typed.lastIndexOf = poison;
check(typed.lastIndexAlias(4, 1) === 0, 'TypedArray lastIndexOf alias');
typed.findAlias = Int16Array.prototype.find; typed.find = poison;
check(typed.findAlias(value => value > 4) === 5, 'TypedArray find alias');
typed.findIndexAlias = Int16Array.prototype.findIndex; typed.findIndex = poison;
check(typed.findIndexAlias(value => value > 4) === 1, 'TypedArray findIndex alias');
typed.findLastAlias = Int16Array.prototype.findLast; typed.findLast = poison;
check(typed.findLastAlias(value => value === 4) === 4, 'TypedArray findLast alias');
typed.findLastIndexAlias = Int16Array.prototype.findLastIndex; typed.findLastIndex = poison;
check(typed.findLastIndexAlias(value => value === 4) === 2, 'TypedArray findLastIndex alias');
typed.everyAlias = Int16Array.prototype.every; typed.every = poison;
check(typed.everyAlias(value => value >= 4), 'TypedArray every alias');
typed.someAlias = Int16Array.prototype.some; typed.some = poison;
check(typed.someAlias(value => value > 4), 'TypedArray some alias');
typed.mapAlias = Int16Array.prototype.map; typed.map = poison;
elements(typed.mapAlias(value => value + 1), [5, 6, 5]);
typed.filterAlias = Int16Array.prototype.filter; typed.filter = poison;
elements(typed.filterAlias(value => value > 4), [5]);
let typedTotal = 0;
typed.eachAlias = Int16Array.prototype.forEach; typed.forEach = poison;
check(typed.eachAlias(value => { typedTotal += value; }) === undefined && typedTotal === 13, 'TypedArray forEach alias');
typed.reduceAlias = Int16Array.prototype.reduce; typed.reduce = poison;
check(typed.reduceAlias((sum, value) => sum + value, 1) === 14, 'TypedArray reduce alias');
typed.reduceRightAlias = Int16Array.prototype.reduceRight; typed.reduceRight = poison;
check(typed.reduceRightAlias((sum, value) => sum + value, 1) === 14, 'TypedArray reduceRight alias');
typed.keysAlias = Int16Array.prototype.keys; typed.keys = poison;
const typedKeys = typed.keysAlias();
check(typedKeys.next().value === 0 && typedKeys.next().value === 1, 'TypedArray keys alias');
typed.entriesAlias = Int16Array.prototype.entries; typed.entries = poison;
elements(typed.entriesAlias().next().value, [0, 4]);
typed.valuesAlias = Int16Array.prototype.values; typed.values = poison;
check(typed.valuesAlias().next().value === 4, 'TypedArray values alias');

const trace = [];
const original = {
  get length() { trace.push('length'); return 2; },
  get 0() { trace.push('get0'); return 'a'; },
  get 1() { trace.push('get1'); return 'b'; },
  join: poison
};
let selected = original;
Object.defineProperty(original, 'alias', {configurable: true, get() {
  trace.push('callee'); return Array.prototype.join;
}});
function base() { trace.push('base'); return selected; }
function key() { trace.push('key'); return 'alias'; }
function argument() {
  trace.push('argument');
  selected = {length: 0};
  Object.defineProperty(original, 'alias', {value: poison});
  return {toString() { trace.push('separator'); return ':'; }};
}
const iterator = {};
let position = 0;
const next = new Proxy(function() {}, {apply(target, receiver, args) {
  check(receiver === iterator && args.length === 0, 'spread next receiver');
  trace.push('next.apply');
  const index = position++;
  check(index < 2, 'bounded spread');
  if (index === 0) Object.defineProperty(iterator, 'next', {value: poison});
  return {
    get done() { trace.push('done' + index); return index === 1; },
    get value() { check(index === 0, 'terminal value unread'); trace.push('value0'); return 'ignored'; }
  };
}});
Object.defineProperty(iterator, 'next', {configurable: true, get() { trace.push('next.get'); return next; }});
Object.defineProperty(iterator, 'return', {get() { throw 'normal argument spread does not close'; }});
const spread = [99];
Object.defineProperty(spread, '0', {get() { throw 'numeric spread snapshot'; }});
Object.defineProperty(spread, Symbol.iterator, {get() {
  trace.push('iterator.get');
  return new Proxy(function() {}, {apply(target, receiver, args) {
    check(receiver === spread && args.length === 0, 'spread open receiver');
    trace.push('iterator.apply'); return iterator;
  }});
}});
function tail() { trace.push('tail'); return 'ignored too'; }
const captured = base()[key()](argument(), ...spread, tail());
check(captured === 'a:b' && selected !== original, 'retained computed Reference');
check(trace.join(',') === 'base,key,callee,argument,iterator.get,iterator.apply,next.get,' +
  'next.apply,done0,value0,next.apply,done1,tail,length,separator,get0,get1', 'full argument and Call order');

const proxyReceiver = {length: 2, 0: 'x', 1: 'y', join: poison};
let applyCalls = 0;
proxyReceiver.alias = new Proxy(Array.prototype.join, {apply(target, receiver, args) {
  check(target === Array.prototype.join && receiver === proxyReceiver && args.length === 3,
    'callable Proxy original receiver and full argc');
  check(args[1] === 'unused' && args[2] === 17, 'ignored arguments retained');
  applyCalls++;
  return Reflect.apply(target, receiver, args);
}});
check(proxyReceiver.alias('-', 'unused', 17) === 'x-y' && applyCalls === 1, 'callable Proxy retained');
print('indexed-collection-reference:ok');
262;
