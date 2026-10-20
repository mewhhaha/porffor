function check(condition, name) { if (!condition) throw name; }
function elements(actual, expected) {
  check(actual.length === expected.length, 'length');
  for (let index = 0; index < expected.length; ++index) check(actual[index] === expected[index], 'element');
}
function poison() { throw 'canonical method must not be selected'; }
const define = Object.defineProperty;
const getPrototypeOf = Object.getPrototypeOf;
const descriptor = Object.getOwnPropertyDescriptor;
const hasOwn = Object.prototype.hasOwnProperty;

const concatTarget = {};
const concat = [4, 5];
concat.alias = Array.prototype.concat; concat.concat = poison;
let concatConstructions = 0;
concat.constructor = {[Symbol.species]: function Species(length) {
  check(length === 0, 'concat species length'); concatConstructions++; return concatTarget;
}};
const concatenated = concat.alias(6);
check(concatenated === concatTarget && !Array.isArray(concatenated) && typeof concatenated === 'object', 'concat arbitrary Object');
elements(concatenated, [4, 5, 6]);
check(concatConstructions === 1, 'one concat species');

function mapTarget() { return 17; }
const map = [4, 5];
map.alias = Array.prototype.map; map.map = poison;
map.constructor = {[Symbol.species]: function Species(length) {
  check(length === 2, 'map species length'); return mapTarget;
}};
const mapped = map.alias(value => value + 1);
check(mapped === mapTarget && typeof mapped === 'function' && mapped() === 17 && !Array.isArray(mapped), 'map arbitrary Function');
check(mapped[0] === 5 && mapped[1] === 6 && mapped.length === 0, 'map writes Function own indices');

function argumentsTarget(first, second) { return arguments; }
const filterTarget = argumentsTarget('old', 'tail');
const filter = [4, 5];
filter.alias = Array.prototype.filter; filter.filter = poison;
filter.constructor = {[Symbol.species]: function Species(length) {
  check(length === 0, 'filter species length'); return filterTarget;
}};
const filtered = filter.alias(value => value > 4);
check(filtered === filterTarget && typeof filtered === 'object' && !Array.isArray(filtered), 'filter arbitrary Arguments');
check(filtered[0] === 5 && filtered[1] === 'tail' && filtered.length === 2, 'filter writes without invented Array length');

const numberString = {alias: Array.prototype.toString, toString: poison, join() { return 23; }};
const objectMarker = {value: 'marker'};
const objectString = {alias: Array.prototype.toString, toString: poison, join() { return objectMarker; }};
function callableMarker() { return 29; }
const functionString = {alias: Array.prototype.toString, toString: poison, join() { return callableMarker; }};
const numeric = numberString.alias();
const object = objectString.alias();
const callable = functionString.alias();
check(typeof numeric === 'number' && numeric + 1 === 24, 'toString arbitrary Number');
check(object === objectMarker && object.value === 'marker', 'toString arbitrary Object');
check(typeof callable === 'function' && callable === callableMarker && callable() === 29, 'toString arbitrary Function');

const genericFill = {length: 2, 0: 1, 1: 2, alias: Array.prototype.fill, fill: poison};
const filled = genericFill.alias('after');
check(filled === genericFill && filled[0] === 'after' && typeof filled[0] === 'string', 'fill result after mutation');
const genericSort = {length: 2, 0: 'z', 1: 'a', alias: Array.prototype.sort, sort: poison};
check(genericSort.alias() === genericSort && genericSort[0] === 'a', 'sort receiver identity');
define(Number.prototype, 'fillAlias', {configurable: true, value: Array.prototype.fill});
define(Number.prototype, 'sortAlias', {configurable: true, value: Array.prototype.sort});
const primitive = 7;
const filledBox = primitive.fillAlias(9);
const sortedBox = primitive.sortAlias();
delete Number.prototype.fillAlias;
delete Number.prototype.sortAlias;
check(typeof filledBox === 'object' && typeof sortedBox === 'object' && filledBox !== sortedBox, 'ToObject primitive results');
check(getPrototypeOf(filledBox) === Number.prototype && getPrototypeOf(sortedBox) === Number.prototype &&
  filledBox.valueOf() === 7 && sortedBox.valueOf() === 7, 'boxed Number identity and value');

const flow = {value: 1};
const getterJoin = {length: 1, get 0() { flow.value = 'getter'; return 'x'; },
  alias: Array.prototype.join, join: poison};
check(getterJoin.alias() === 'x' && typeof flow.value === 'string' && flow.value.slice(0, 3) === 'get', 'getter effect invalidates shape');
let live = 1;
const separator = {toString() { live = function() { return 31; }; return ':'; }};
const coercingJoin = {length: 2, 0: 'a', 1: 'b', alias: Array.prototype.join, join: poison};
check(coercingJoin.alias(separator) === 'a:b' && typeof live === 'function' && live() === 31, 'coercion effect invalidates captured value');
const callbackFlow = {value: 2};
const callbackMap = {length: 1, 0: 4, alias: Array.prototype.map, map: poison};
const callbackResult = callbackMap.alias(value => { callbackFlow.value = 'callback'; return value + 1; });
check(callbackResult[0] === 5 && typeof callbackFlow.value === 'string' && callbackFlow.value.charAt(0) === 'c', 'callback effect invalidates shape');

function compareSorted(a, b) {
  'use strict';
  if (a === undefined && b === undefined) return 0;
  check(typeof a === 'number' && typeof b === 'number' && this === undefined, 'toSorted comparator observed inputs');
  return a - b;
}
compareSorted();
const mixedSorted = {length: 3, 0: 3, 1: 1, 2: 2, alias: Array.prototype.toSorted, toSorted: poison};
elements(mixedSorted.alias(compareSorted), [1, 2, 3]);
function compareSort(a, b) {
  'use strict';
  if (a === undefined && b === undefined) return 0;
  check(typeof a === 'number' && typeof b === 'number' && this === undefined, 'sort comparator observed inputs');
  return a - b;
}
compareSort();
const mixedSort = {length: 3, 0: 3, 1: 1, 2: 2, alias: Array.prototype.sort, sort: poison};
check(mixedSort.alias(compareSort) === mixedSort, 'mixed-use sort identity');
elements(mixedSort, [1, 2, 3]);

const callbackThis = {};
const typed = new Int16Array([4, 5, 6]);
function predicate(value, index, source) {
  'use strict';
  if (value === undefined) return false;
  check(typeof value === 'number' && typeof index === 'number' && source === typed && this === callbackThis,
    'TypedArray callback prior omitted-call facts');
  return value > 4;
}
predicate();
typed.findAlias = Int16Array.prototype.find; typed.find = poison;
check(typed.findAlias(predicate, callbackThis) === 5, 'mixed-use TypedArray find');
typed.findIndexAlias = Int16Array.prototype.findIndex; typed.findIndex = poison;
check(typed.findIndexAlias(predicate, callbackThis) === 1, 'mixed-use TypedArray findIndex');
typed.findLastAlias = Int16Array.prototype.findLast; typed.findLast = poison;
check(typed.findLastAlias(predicate, callbackThis) === 6, 'mixed-use TypedArray findLast');
typed.findLastIndexAlias = Int16Array.prototype.findLastIndex; typed.findLastIndex = poison;
check(typed.findLastIndexAlias(predicate, callbackThis) === 2, 'mixed-use TypedArray findLastIndex');
typed.everyAlias = Int16Array.prototype.every; typed.every = poison;
check(typed.everyAlias(predicate, callbackThis) === false, 'mixed-use TypedArray every');
typed.someAlias = Int16Array.prototype.some; typed.some = poison;
check(typed.someAlias(predicate, callbackThis) === true, 'mixed-use TypedArray some');

const arrayIteratorPrototype = getPrototypeOf([].values());
const stringIteratorPrototype = getPrototypeOf('a'[Symbol.iterator]());
check(arrayIteratorPrototype !== stringIteratorPrototype, 'String iterator distinct prototype');
const oldArrayNext = descriptor(arrayIteratorPrototype, 'next');
const oldArrayIterator = descriptor(arrayIteratorPrototype, Symbol.iterator);
const oldStringNext = descriptor(stringIteratorPrototype, 'next');
const oldStringIterator = descriptor(stringIteratorPrototype, Symbol.iterator);
const token = {};
let arrayNextCalls = 0;
let stringNextCalls = 0;
define(arrayIteratorPrototype, 'next', {configurable: true, writable: true, value: function(argument) {
  check(getPrototypeOf(this) === arrayIteratorPrototype && argument === token && arguments.length === 1, 'mutated Array iterator Call');
  arrayNextCalls++; return 41;
}});
define(arrayIteratorPrototype, Symbol.iterator, {configurable: true, writable: true, value: function(argument) {
  check(argument === token && arguments.length === 1, 'mutated Array iterator Symbol Call'); return this;
}});
define(stringIteratorPrototype, 'next', {configurable: true, writable: true, value: function(argument) {
  check(getPrototypeOf(this) === stringIteratorPrototype && argument === token && arguments.length === 1, 'mutated String iterator Call');
  stringNextCalls++; return 43;
}});
define(stringIteratorPrototype, Symbol.iterator, {configurable: true, writable: true, value: function(argument) {
  check(argument === token && arguments.length === 1, 'mutated String iterator Symbol Call'); return this;
}});
function inspectArrayIterator(iterator) {
  check(getPrototypeOf(iterator) === arrayIteratorPrototype && !hasOwn.call(iterator, 'next') &&
    !hasOwn.call(iterator, Symbol.iterator), 'iterator methods inherited, not fabricated own data');
  const result = iterator.next(token);
  check(typeof result === 'number' && result + 1 === 42, 'fresh iterator observes mutated next result');
  const saved = iterator[Symbol.iterator];
  iterator.alias = saved;
  check(iterator.alias(token) === iterator, 'mutated Symbol iterator retained alias');
}
const freshKeys = {length: 1, alias: Array.prototype.keys};
inspectArrayIterator(freshKeys.alias());
const freshEntries = {length: 1, 0: 9, alias: Array.prototype.entries};
inspectArrayIterator(freshEntries.alias());
const freshValues = {length: 1, 0: 9, alias: Array.prototype.values};
inspectArrayIterator(freshValues.alias());
typed.keysAlias = Int16Array.prototype.keys;
inspectArrayIterator(typed.keysAlias());
typed.entriesAlias = Int16Array.prototype.entries;
inspectArrayIterator(typed.entriesAlias());
typed.valuesAlias = Int16Array.prototype.values;
inspectArrayIterator(typed.valuesAlias());
const stringSource = {alias: String.prototype[Symbol.iterator], toString() { return 'ab'; }};
const stringIterator = stringSource.alias();
check(getPrototypeOf(stringIterator) === stringIteratorPrototype && !hasOwn.call(stringIterator, 'next') &&
  !hasOwn.call(stringIterator, Symbol.iterator), 'String iterator own-property honesty');
const stringResult = stringIterator.next(token);
check(typeof stringResult === 'number' && stringResult + 1 === 44 && stringIterator[Symbol.iterator](token) === stringIterator,
  'String iterator uses its mutable prototype');
define(arrayIteratorPrototype, 'next', oldArrayNext);
if (oldArrayIterator === undefined) delete arrayIteratorPrototype[Symbol.iterator];
else define(arrayIteratorPrototype, Symbol.iterator, oldArrayIterator);
define(stringIteratorPrototype, 'next', oldStringNext);
if (oldStringIterator === undefined) delete stringIteratorPrototype[Symbol.iterator];
else define(stringIteratorPrototype, Symbol.iterator, oldStringIterator);
check(arrayNextCalls === 6 && stringNextCalls === 1, 'all seven fresh iterator factories');
const restoredString = '😀a'[Symbol.iterator]();
check(restoredString.next().value === '😀' && restoredString.next().value === 'a' && restoredString.next().done,
  'String codepoint iterator restored');
print('indexed-collection-results:ok');
262;
