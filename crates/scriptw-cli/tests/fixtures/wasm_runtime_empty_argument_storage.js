function check(value, message) { if (!value) throw new Error(message); }
let invocations = 0;
function collect() { ++invocations; return arguments; }
const events = [];
function empty() {
  return { [Symbol.iterator]() {
    events.push('iterator');
    return { next() {
      events.push('next');
      return { done: true, get value() { throw new Error('done iterator value'); } };
    } };
  } };
}
const first = collect(...empty());
const second = collect(...empty(), ...empty());
check(first !== second && first.length === 0 && second.length === 0, 'fresh empty arguments');
first[0] = 'first'; first.extra = 1;
check(second[0] === undefined && second.extra === undefined, 'arguments remain independent');
check(events.join(',') === 'iterator,next,iterator,next,iterator,next', 'iterator effects');
const arrayLike = { get length() { events.push('length'); return 0; }, get 0() { throw new Error('zero index read'); } };
const applied = Reflect.apply(collect, null, arrayLike);
check(applied !== first && applied.length === 0 && applied[0] === undefined, 'reflect empty list');
check(events[events.length - 1] === 'length', 'length getter retained');
const marker = {};
try { collect(...{ [Symbol.iterator]() { return { next() { throw marker; } }; } }); throw new Error('missed throw'); }
catch (e) { check(e === marker, 'iterator throw'); }
check(invocations === 3, 'abrupt spread never calls');
function rest(...args) { return args; }
const rest1 = rest(...empty());
const rest2 = rest(...empty());
rest1.push('changed');
check(rest1 !== rest2 && rest2.length === 0, 'rest storage remains fresh');
const bound = collect.bind(null, ...empty());
const bound1 = bound(); const bound2 = bound();
bound1.extra = true;
check(bound1 !== bound2 && bound2.extra === undefined && bound2.length === 0, 'bound empty arguments');
function Base() { this.args = arguments; }
class Derived extends Base { constructor(...values) { super(...values); } }
const made1 = new Derived(...empty());
const made2 = Reflect.construct(Derived, arrayLike);
made1.args[0] = 'changed';
check(made1 !== made2 && made1.args !== made2.args && made2.args.length === 0 && made2.args[0] === undefined, 'constructor empty arguments');
function* generator(...values) { yield values; return arguments; }
const gen1 = generator(...empty()); const gen2 = generator(...empty());
const yielded1 = gen1.next().value; const yielded2 = gen2.next().value;
yielded1.push('changed');
check(yielded1 !== yielded2 && yielded2.length === 0, 'generator empty rest');
const done1 = gen1.next().value; const done2 = gen2.next().value;
done1.extra = true;
check(done1 !== done2 && done2.extra === undefined, 'generator empty arguments');
const unusual = [];
unusual[Symbol.iterator] = function* () { yield 17; };
check(collect(...unusual)[0] === 17, 'custom iterator on empty array');
for (let i = 0; i < 1300; ++i) {
  const args = collect(...(i % 2 ? [] : [i, marker]));
  check(args.length === (i % 2 ? 0 : 2), 'mixed lengths');
  if (i % 2) { args[0] = i; check(collect(...[])[0] === undefined, 'no stale empty storage'); }
  else check(args[0] === i && args[1] === marker, 'nonempty argument preservation');
}
const lengthEffects = [];
const convertedEmpty = { get length() {
  lengthEffects.push('length');
  return { [Symbol.toPrimitive](hint) { lengthEffects.push(hint); return NaN; } };
}, get 0() { throw new Error('converted-empty index'); } };
check(Reflect.apply(collect, null, convertedEmpty).length === 0, 'NaN length is empty');
check(lengthEffects.join(',') === 'length,number', 'length conversion ordering');
const lengthMarker = {};
try {
  Reflect.apply(collect, null, { length: { valueOf() { throw lengthMarker; } } });
  throw new Error('missed length throw');
} catch (e) { check(e === lengthMarker, 'abrupt length conversion'); }
const captures = [];
function capture(...values) { return collect.bind(null, ...values); }
for (let i = 0; i < 24; ++i) captures.push(capture(...(i % 2 ? [] : [i, marker])));
if (typeof gc === 'function') gc();
for (let i = 0; i < captures.length; ++i) {
  const args = captures[i]();
  check(args.length === (i % 2 ? 0 : 2), 'captured length');
  if (!(i % 2)) check(args[0] === i && args[1] === marker, 'captured nonempty values');
}
print('runtime-empty-argument-storage-ok');
