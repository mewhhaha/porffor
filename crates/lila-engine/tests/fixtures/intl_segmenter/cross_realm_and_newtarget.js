function check(value, message) { if (!value) throw new Error(message); }
var foreign = __lilaCreateRealm().global;
var foreignConstructor = foreign.Intl.Segmenter;
var result = new foreignConstructor('en');
check(Object.getPrototypeOf(result) === foreignConstructor.prototype, 'foreign Segmenter prototype');
check(Object.getPrototypeOf(result.resolvedOptions()) === foreign.Object.prototype, 'foreign method result realm');
var current = new Intl.Segmenter('en'), segments = foreignConstructor.prototype.segment.call(current, 'AB');
var foreignSegmentsPrototype = Object.getPrototypeOf(new foreignConstructor().segment('AB'));
check(Object.getPrototypeOf(segments) === foreignSegmentsPrototype, 'segment called-function realm');
check(Object.getPrototypeOf(segments.containing(0)) === foreign.Object.prototype, 'containing result realm');
var iterator = segments[Symbol.iterator]();
check(Object.getPrototypeOf(iterator.next()) === foreign.Object.prototype, 'next result realm');
// Binding a foreign ordinary function preserves its foreign [[Realm]] while
// allowing a legal primitive prototype result (Function.prototype is immutable).
var target = foreign.Function.bind(null);
Object.defineProperty(target, 'prototype', { value: 1, configurable: true });
var fallback = Reflect.construct(Intl.Segmenter, ['sr'], target);
check(Object.getPrototypeOf(fallback) === foreignConstructor.prototype, 'GetFunctionRealm primitive fallback');
function Sub() {} var custom = Object.create(null); Sub.prototype = custom;
var customResult = Reflect.construct(Intl.Segmenter, ['en'], Sub);
check(Object.getPrototypeOf(customResult) === custom, 'null-prototype object NewTarget');
print('ok cross_realm_and_newtarget'); 262;
