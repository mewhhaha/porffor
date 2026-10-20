function check(condition, label) { if (!condition) throw label; }
var whole = { marker: 79 }; whole.self = whole;
whole[Symbol.toPrimitive] = function () { throw 'whole-was-coerced'; };
function source(value, events, closeError) {
  var record = { next: function () { events.push('next'); return { done: false, value: value }; }, return: function () { events.push('close'); if (closeError) throw closeError; return {}; } }, input = {};
  input[Symbol.iterator] = function () { events.push('iterator'); return record; };
  return input;
}

async function run() {
  var events = [], box = {}, other = {}, rawKey = { [Symbol.toPrimitive]: function () { events.push('key-coerce'); return 'chosen'; } };
  async function* members(input) { return ([(yield 'base')[yield 'raw-key'] = await (yield 'default')] = input); }
  var input = source(undefined, events), iterator = members(input);
  check((await iterator.next()).value === 'base', 'member-base-before-iterator-step');
  check(events.join(',') === 'iterator', 'no-next-before-member-reference');
  check((await iterator.next(box)).value === 'raw-key', 'raw-computed-reference-key');
  check((await iterator.next(rawKey)).value === 'default', 'step-before-lazy-default');
  check(events.join(',') === 'iterator,next', 'raw-key-not-coerced-before-default');
  gc(); var result = await iterator.next(31);
  check(result.done === true && result.value === input && box.chosen === 31 && other.chosen === undefined, 'original-raw-rhs-and-captured-member');
  check(events.join(',') === 'iterator,next,key-coerce,close', 'original-put-coercion-then-close');

  var reads = 0, writes = 0, original = { get selected() { reads++; return 1; }, set selected(value) { writes++; this.received = value; } }, alternate = { selected: 0 };
  var scope = { selected: 0 }, inputEvents = [];
  var proxy = new Proxy(original, { has: function (object, key) { return key === 'selected'; } });
  async function* selected(input) { with (proxy) { ([selected = await (yield 'reference')] = input); } return original.received; }
  iterator = selected(source(undefined, inputEvents)); check((await iterator.next()).value === 'reference', 'write-only-identifier-reference-before-default');
  original[Symbol.unscopables] = { selected: true }; gc();
  check((await iterator.next(37)).value === 37 && writes === 1 && reads === 0, 'original-selected-object-record-held-without-get');
  check(scope.selected === 0 && alternate.selected === 0, 'no-reference-reselection-after-unscopables');

  events = []; var rejected = source(undefined, events, { close: true });
  async function* defaults(input) { var received; return ([received = await (yield 'reject')] = input); }
  iterator = defaults(rejected); await iterator.next();
  try { await iterator.next(Promise.reject(whole)); throw 'missing-rejected-default'; } catch (error) { check(error === whole && events[events.length - 1] === 'close', 'rejected-await-closes-with-original-throw-precedence'); }
  events = []; var closeError = { close: 83 }; iterator = defaults(source(undefined, events, closeError)); await iterator.next();
  try { await iterator.return(whole); throw 'missing-close-error'; } catch (error) { check(error === closeError, 'return-close-error-precedence'); }

  var valueGets = 0, nextCalls = 0, finite = {};
  finite[Symbol.iterator] = function () { return { next: function () { nextCalls++; if (nextCalls > 3) return { done: true }; return { done: false, get value() { valueGets++; return nextCalls; } }; } }; };
  async function* rest(input) { var received, tail; ([, received = await (yield 'never'), ...tail] = input); yield [received, tail]; }
  iterator = rest(finite); result = await iterator.next();
  check(result.value[0] === 2 && result.value[1].join(',') === '3' && valueGets === 2 && nextCalls === 4, 'elision-skips-value-and-rest-drains-original-record');
  check((await iterator.next()).done === true, 'rest-pattern-completes');

  async function* eager(input) { var received; return ([received] = await (yield input)); }
  iterator = eager([5]); var yielded = await iterator.next(); result = await iterator.next(yielded.value);
  check(result.done === true && result.value === yielded.value, 'eager-pattern-mixed-rhs-preserves-whole-result');

  var superEvents = [], writes = [];
  class Parent { set chosen(value) { superEvents.push('original-put'); writes.push([this, value]); } }
  class Alternate { set chosen(value) { throw 'reselected-super-base'; } }
  class Child extends Parent {
    async *array(input) { return ([super[await (yield 'super-key')] = await (yield 'super-default')] = input); }
    async *object(input) { return ({ value: super[await (yield 'object-super-key')] = await (yield 'object-super-default') } = input); }
  }
  var child = new Child(), superInput = source(undefined, superEvents);
  iterator = child.array(superInput); check((await iterator.next()).value === 'super-key', 'super-target-after-acquisition');
  var superKey = { [Symbol.toPrimitive]: function () { superEvents.push('super-key-coerce'); return 'chosen'; } };
  check((await iterator.next(superKey)).value === 'super-default', 'super-reference-held-before-default');
  check(superEvents.join(',') === 'iterator,next', 'write-only-super-reference-does-not-get-or-coerce');
  Object.setPrototypeOf(Child.prototype, Alternate.prototype); gc();
  result = await iterator.next(whole);
  check(result.done && result.value === superInput && writes.length === 1 && writes[0][0] === child && writes[0][1] === whole, 'super-pattern-retains-original-base-this-and-whole-value');
  check(superEvents.join(',') === 'iterator,next,super-key-coerce,original-put,close', 'super-put-then-original-iterator-close');
  Object.setPrototypeOf(Child.prototype, Parent.prototype);
  superEvents = []; var objectInput = { get value() { superEvents.push('source-get'); return undefined; } };
  iterator = child.object(objectInput); await iterator.next();
  check((await iterator.next('chosen')).value === 'object-super-default' && superEvents.join(',') === 'source-get', 'object-super-reference-before-source-get-and-default');
  Object.setPrototypeOf(Child.prototype, Alternate.prototype); gc();
  result = await iterator.next(47);
  check(result.done && result.value === objectInput && writes[1][0] === child && writes[1][1] === 47, 'object-super-pattern-spends-original-reference');
  Object.setPrototypeOf(Child.prototype, Parent.prototype);
  class Throwing { set chosen(value) { throw whole; } }
  Object.setPrototypeOf(Child.prototype, Throwing.prototype); superEvents = [];
  iterator = child.array(source(undefined, superEvents, { close: true })); await iterator.next(); await iterator.next('chosen');
  try { await iterator.next(1); throw 'missing-super-setter-throw'; } catch (error) {
    check(error === whole && superEvents[superEvents.length - 1] === 'close', 'super-target-throw-closes-and-wins-close-error');
  }
  Object.setPrototypeOf(Child.prototype, Parent.prototype);

  var resolve, promise = new Promise(function (done) { resolve = done; }), queuedEvents = [];
  async function* queue(input) { try { var [received = await (yield 'queue')] = input; yield received; } finally { await Promise.resolve(0); yield 'finally'; } }
  iterator = queue(source(undefined, queuedEvents)); await iterator.next();
  var next = iterator.next(promise), queuedReturn = iterator.return(whole); gc(); resolve(43);
  check((await next).value === 43, 'queued-next-resumes-complete-default');
  check((await queuedReturn).value === 'finally' && queuedEvents[queuedEvents.length - 1] === 'close', 'queued-return-runs-original-finalizer');
  result = await iterator.next(); check(result.done === true && result.value === whole, 'queued-whole-return-after-finally');
}
run().then(function () { print('mixed-async-generator-pattern-assignments:ok'); }, function (error) { print(error); throw error; });
