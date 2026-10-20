function check(condition, label) { if (!condition) throw label; }
var whole = { tag: 'whole' }; whole.self = whole;
async function run() {
  var events = [], closes = 0, source = {}, keyGets = 0, spreadGets = 0, proto = { inherited: 19 };
  source[Symbol.iterator] = function () {
    events.push('iterator'); var index = 0;
    return { next: function () { events.push('next'); index++; return index < 3 ? { done: false, value: index + 1 } : { done: true }; }, return: function () { closes++; return {}; } };
  };
  function mark(name, value) { events.push(name); return value; }
  var rawKey = { [Symbol.toPrimitive]: function () { keyGets++; events.push('key'); return 'selected'; } };
  var copied = { get copied() { spreadGets++; events.push('copy'); return whole; } };
  async function* literals() {
    const array = [mark('first', 1), ...source, await (yield 'array'), , mark('last', 5)];
    const object = { [(yield 'key', await Promise.resolve(rawKey))]: yield array, __proto__: proto, method() { return this; }, get own() { return 7; }, ...copied };
    yield object;
  }
  var iterator = literals();
  check((await iterator.next()).value === 'array', 'literal-prefix-before-first-suspension');
  check(events.join(',') === 'first,iterator,next,next,next' && closes === 0, 'spread-drained-once-before-later-await');
  check((await iterator.next(4)).value === 'key', 'await-only-element-completes-before-next-yield');
  gc(); var array = (await iterator.next()).value;
  check(array.length === 6 && array[0] === 1 && array[1] === 2 && array[2] === 3 && array[3] === 4 && !(4 in array) && array[5] === 5, 'one-accumulator-preserves-elision-and-prefix');
  check(events.join(',') === 'first,iterator,next,next,next,last,key' && keyGets === 1 && spreadGets === 0, 'computed-key-coercion-before-value');
  array[0] = 41; gc(); var object = (await iterator.next(whole)).value;
  check(object.selected === whole && object.copied === whole && object.inherited === 19 && object.method() === object && object.own === 7, 'original-object-definition-prototype-method-and-spread');
  check(array[0] === 41 && spreadGets === 1 && keyGets === 1 && closes === 0, 'original-retained-identities-without-repeated-source');
  check((await iterator.next()).done, 'literal-owner-completes');

  events = []; function Base() {} Base.prototype.base = 23;
  var classKey = { [Symbol.toPrimitive]: function () { events.push('class-key'); return 'read'; } };
  async function* classes() {
    const C = class Inner extends (await (yield Base)) { [(yield 'computed', await Promise.resolve(classKey))]() { return Inner; } static label = this.name; };
    yield C;
    const Inferred = class extends (await (yield Base)) { static label = this.name; static read() { return Inferred; } };
    yield Inferred;
  }
  iterator = classes(); check((await iterator.next()).value === Base, 'class-heritage-before-computed-names');
  check((await iterator.next(Base)).value === 'computed', 'completed-heritage-retained-before-key');
  gc(); var C = (await iterator.next()).value;
  check(C.name === 'Inner' && C.label === 'Inner' && new C().read() === C && new C().base === 23 && events.join(',') === 'class-key', 'original-explicit-class-name-environment-and-heritage');
  check((await iterator.next()).value === Base, 'inferred-class-second-heritage');
  var Inferred = (await iterator.next(Base)).value;
  check(Inferred.name === 'Inferred' && Inferred.label === 'Inferred' && Inferred.read() === Inferred, 'inferred-label-retains-outer-binding');
  check((await iterator.next()).done, 'class-owner-completes');

  events = []; var failure = {};
  failure[Symbol.iterator] = function () { return { next: function () { throw whole; }, return: function () { events.push('close'); return {}; } }; };
  async function* failedSpread() { try { yield [...failure, await (yield 'unreached')]; } catch (error) { check(error === whole, 'whole-original-step-failure'); yield 'caught'; } }
  iterator = failedSpread(); check((await iterator.next()).value === 'caught' && events.length === 0, 'literal-spread-step-error-never-enters-later-protocol');
  await iterator.next();
}
run().then(function () { print('mixed-async-generator-literals-classes:ok'); }, function (error) { print(error); throw error; });
