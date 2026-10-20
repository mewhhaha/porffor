function check(condition, label) { if (!condition) throw label; }
var whole = { tag: 'whole' }; whole.self = whole;
async function run() {
  var effects = 0, getterCalls = 0, callThis, original = { marker: 29 }, alternate = { marker: 31 }, key;
  function effect() { effects++; throw 'shorted-tail-was-evaluated'; }
  Object.defineProperty(original, 'method', { configurable: true, get: function () { getterCalls++; return function (value) { callThis = this; return [this.marker, value]; }; } });
  async function* skipped() { const value = (yield null)?.[await effect()]?.method(yield effect(), ...effect()); yield value; }
  var iterator = skipped(); check((await iterator.next()).value === null, 'actual-nullish-base-yields-once');
  check((await iterator.next()).value === undefined && effects === 0, 'all-shorted-key-call-spread-and-yield-skipped');
  check((await iterator.next()).done, 'shorted-chain-completes');

  async function* grouped() { const result = ((yield original)?.[await (yield 'key')])(await (yield 'argument')); yield result; }
  iterator = grouped(); check((await iterator.next()).value === original, 'grouped-base-identity');
  check((await iterator.next(original)).value === 'key' && getterCalls === 0, 'property-key-completes-before-get');
  check((await iterator.next('method')).value === 'argument' && getterCalls === 1, 'one-get-before-outer-argument');
  Object.defineProperty(original, 'method', { configurable: true, value: function () { throw 'callee-reselected'; } });
  original.marker = 37; gc(); var result = (await iterator.next(whole)).value;
  check(result[0] === 37 && result[1] === whole && callThis === original && getterCalls === 1, 'retained-grouped-reference-receiver-and-callee');
  check((await iterator.next()).done && alternate.marker === 31, 'grouped-reference-completes');

  var gets = 0, deletes = 0, coercions = 0, thenGets = 0, target = { selected: 43 };
  var proxy = new Proxy(target, { get: function (object, name) {
    // Plain async-generator yield awaits its value before publishing it.
    // That required thenable probe precedes Delete Reference acquisition.
    if (name === 'then') { thenGets++; return undefined; }
    gets++; throw 'delete-invoked-get';
  }, deleteProperty: function (object, name) { deletes++; return Reflect.deleteProperty(object, name); } });
  key = { [Symbol.toPrimitive]: function () { coercions++; return 'selected'; } };
  async function* deleted() { const value = delete ((yield proxy)?.[await (yield 'delete-key')]); yield value; }
  iterator = deleted(); check((await iterator.next()).value === proxy, 'delete-base-is-retained');
  check(thenGets === 1 && gets === 0 && deletes === 0, 'yield-await-probes-then-before-delete-reference');
  check((await iterator.next(proxy)).value === 'delete-key', 'delete-key-source-suspends');
  gc(); check((await iterator.next(key)).value === true && gets === 0 && deletes === 1 && coercions === 1 && !('selected' in target), 'terminal-delete-reference-no-get-and-once-only-coercion');
  check(thenGets === 1, 'delete-does-not-repeat-the-yield-await-probe');
  await iterator.next();
  async function* shortDelete() { const value = delete ((yield null)?.[await effect()]); yield value; }
  iterator = shortDelete(); await iterator.next(); check((await iterator.next()).value === true && effects === 0, 'shorted-delete-publishes-true-without-key'); await iterator.next();

  target = { selected: 47 }; deletes = 0; coercions = 0;
  async function* rawDelete() { const value = delete (yield target)[await (yield 'raw-key')]; yield value; }
  iterator = rawDelete(); await iterator.next(); check((await iterator.next(target)).value === 'raw-key', 'raw-delete-original-reference-source');
  check((await iterator.next(key)).value === true && coercions === 1 && !('selected' in target), 'raw-delete-await-key-without-get'); await iterator.next();

  async function* nullOuter() { try { ((yield null)?.[await effect()])(await (yield 'outer')); } catch (error) { check(error instanceof TypeError, 'grouped-nullish-outer-call-still-throws'); yield 'caught'; } }
  iterator = nullOuter(); await iterator.next(); check((await iterator.next()).value === 'outer' && effects === 0, 'grouping-keeps-outer-argument-outside-short-chain');
  check((await iterator.next(1)).value === 'caught', 'outer-call-after-completed-argument'); await iterator.next();

  var resolve, promise = new Promise(function (done) { resolve = done; });
  async function* queued() { try { const value = (yield original)?.[await (yield 'queued-key')]; yield value; } finally { await Promise.resolve(); gc(); yield 'finally'; } }
  iterator = queued(); await iterator.next(); await iterator.next(original);
  var next = iterator.next(promise), returned = iterator.return(whole); gc(); resolve('marker');
  check((await next).value === 37, 'normal-await-selected-tail-before-queued-return');
  check((await returned).value === 'finally', 'whole-return-keeps-original-yielding-finalizer');
  result = await iterator.next(); check(result.done && result.value === whole, 'whole-return-after-mixed-tail-finalizer');
}
run().then(function () { print('mixed-async-generator-optional-delete:ok'); }, function (error) { print(error); throw error; });
