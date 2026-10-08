function check(condition, label) { if (!condition) throw label; }
var whole = { marker: 73 }; whole.self = whole;
whole[Symbol.toPrimitive] = function () { throw 'unexpected-conversion'; };
var value = 'outside';

async function references() {
  var events = [], views = { value: 3 }, excluded = { value: false };
  views[Symbol.unscopables] = excluded;
  var scope = new Proxy(views, {
    has: function (object, key) { if (key === 'value') events.push('has'); return Reflect.has(object, key); },
    get: function (object, key, receiver) { if (key === 'value') events.push('get'); return Reflect.get(object, key, receiver); },
    set: function (object, key, next, receiver) { if (key === 'value') events.push('set'); return Reflect.set(object, key, next, receiver); }
  });
  async function* selecting() { with(scope) { for(value in await (yield 'keys-head')) { yield 'written'; await Promise.resolve(0); } } }
  var iterator = selecting(); check((await iterator.next()).value === 'keys-head', 'per-key-reference-head');
  check((await iterator.next({ first: 1, second: 2 })).value === 'written' && views.value === 'first', 'first-original-eager-with-reference');
  excluded.value = true; gc();
  check((await iterator.next()).value === 'written' && value === 'second' && views.value === 'first', 'per-key-reference-reselects-original-record');
  check((await iterator.next()).done === true, 'per-key-reference-loop-completes');
  excluded.value = false; views.value = 3; events = [];
  async function* compound() { with(scope) { for(const key in await (yield 'compound-head')) { value += await (yield 'rhs'); yield value; } } }
  iterator = compound(); check((await iterator.next()).value === 'compound-head', 'compound-head');
  check((await iterator.next({ only: 1 })).value === 'rhs', 'compound-reference-before-rhs');
  check(events.join(',') === 'has,has,get', 'selected-object-reference-hasbinding-and-getbindingvalue');
  excluded.value = true; views.value = 99; gc();
  check((await iterator.next(4)).value === 'second' && views.value === 7, 'compound-retains-original-reference-and-old-value');
  check(events.filter(function (event) { return event === 'set'; }).length === 1, 'compound-put-exactly-once');
  check((await iterator.next()).done === true, 'compound-reference-loop-completes');
  excluded.value = false; events = [];
  async function* plain() { with(scope) { for(const key in await (yield 'plain-head')) { value = await (yield 'plain-rhs'); } } }
  iterator = plain(); await iterator.next(); await iterator.next({ only: 1 });
  check(events.join(',') === 'has', 'plain-reference-before-rhs-without-get');
  excluded.value = true; gc();
  check((await iterator.next(whole)).done === true && views.value === whole && value === 'second', 'plain-put-retains-whole-value-and-original-record');
  check(events.filter(function (event) { return event === 'get'; }).length === 0, 'plain-write-reference-never-gets');
  excluded.value = false; views.value = 0; events = []; var skipped = 0;
  async function* logical() { with(scope) { for(const key in await (yield 'logical-head')) { value &&= await { get then() { skipped++; throw whole; } }; yield key; } } }
  iterator = logical(); await iterator.next();
  check((await iterator.next({ only: 1 })).value === 'only' && skipped === 0 && views.value === 0, 'skipped-logical-arm-never-adopts-rhs');
  check(events.join(',') === 'has,has,get', 'skipped-logical-reference-has-no-put');
  check((await iterator.next()).done === true, 'skipped-logical-reference-released');
  excluded.value = false; events = [];
  async function* caught() { with(scope) { for(const key in await (yield 'caught-head')) { try { value = await (yield 'abandon'); } catch(error) { check(error === whole, 'injected-whole-throw'); value = await (yield 'fresh'); } } } }
  iterator = caught(); await iterator.next(); await iterator.next({ only: 1 });
  check((await iterator.throw(whole)).value === 'fresh', 'catch-after-abandoned-reference');
  check((await iterator.next(11)).done === true && views.value === 11, 'fresh-selected-write-after-injected-completion');
  check(events.filter(function (event) { return event === 'set'; }).length === 1, 'abandoned-reference-never-publishes');
  var calls = 0, puts = [];
  function selectedTarget() { calls++; return { set key(next) { puts.push(next); } }; }
  async function* property() { for(selectedTarget().key in await (yield 'property-head')) { yield 'property-body'; await Promise.resolve(0); } }
  iterator = property(); await iterator.next(); await iterator.next({ aa: 1, bb: 2 }); await iterator.next();
  check((await iterator.next()).done === true && calls === 2 && puts.join(',') === 'aa,bb', 'original-property-target-evaluated-once-per-key');
  var original = async function* immutable() { for(immutable in await (yield 'immutable-head')) { yield 'immutable-body'; } return immutable; };
  iterator = original(); await iterator.next(); check((await iterator.next({ only: 1 })).value === 'immutable-body', 'actual-sloppy-ignored-immutable-head-write');
  check((await iterator.next()).value === original, 'ignored-write-keeps-original-named-function-binding');
  var strict = async function* immutable() { 'use strict'; try { for(immutable in await (yield 'strict-head')) { throw 'entered-strict-body'; } } catch(error) { check(error instanceof TypeError, 'strict-immutable-head-write-throws'); return whole; } };
  iterator = strict(); await iterator.next();
  check((await iterator.next({ only: 1 })).value === whole, 'strict-head-cleanup-preserves-whole-return');
  var superWrites = [];
  class SuperParent { set chosen(key) { superWrites.push([this, key]); } }
  class SuperChild extends SuperParent { async *values(input) { for(super.chosen in await (yield input)) { yield this; } } }
  var receiver = new SuperChild(); iterator = receiver.values({ first: 1, second: 2 });
  var head = await iterator.next();
  check((await iterator.next(head.value)).value === receiver && superWrites[0][0] === receiver && superWrites[0][1] === 'first', 'eager-super-head-publishes-original-selected-key');gc();
  check((await iterator.next()).value === receiver && superWrites[1][1] === 'second', 'eager-super-reference-evaluates-on-each-key');
  check((await iterator.next()).done, 'eager-super-enumeration-completes');
  var heldKeys = [], target = { set chosen(key) { heldKeys.push(key); } }, input = { first: 1, second: 2 };
  async function* suspendedTarget() { for((yield 'target')[await (yield 'raw-key')] in input) { yield 'put'; } }
  iterator = suspendedTarget(); check((await iterator.next()).value === 'target', 'selected-key-before-suspended-target');
  check((await iterator.next(target)).value === 'raw-key', 'per-key-target-base-held-before-key');gc();
  var keyCoercions = 0, rawKey = { [Symbol.toPrimitive]: function () { keyCoercions++; return 'chosen'; } };
  delete input.second;
  check((await iterator.next(rawKey)).value === 'put' && heldKeys.join(',') === 'first' && keyCoercions === 1, 'selected-string-key-held-through-complete-target');
  check((await iterator.next()).done && heldKeys.length === 1, 'next-enumeration-still-checks-deleted-descriptor');
  async function* suspendedPattern(input) { for(const {absent:received=await (yield function read(){return received;})} in input) { yield function read(){return received;}; } }
  iterator = suspendedPattern({ first: 1, second: 2 }); var headRead = (await iterator.next()).value;
  try { headRead(); throw 'initialized-default-tdz'; } catch(error) { check(error instanceof ReferenceError, 'original-pattern-record-tdz-during-default'); }
  gc(); var firstRead = (await iterator.next(whole)).value;
  check(firstRead() === whole && headRead() === whole, 'default-initializes-original-captured-per-key-cell');
  var secondHead = (await iterator.next()).value;
  try { secondHead(); throw 'initialized-second-default-tdz'; } catch(error) { check(error instanceof ReferenceError, 'each-key-has-fresh-original-pattern-record'); }
  var secondRead = (await iterator.next(19)).value;
  check(secondRead() === 19 && firstRead() === whole, 'separate-key-closures-survive-resumption');
  check((await iterator.next()).done, 'suspended-pattern-enumeration-completes');
  class SuspendedSuperChild extends SuperParent { async *values(input) { for(super[await (yield 'super-key')] in input) { yield this; } } }
  receiver = new SuspendedSuperChild(); iterator = receiver.values({ only: 1 });
  check((await iterator.next()).value === 'super-key', 'suspended-super-key-after-selection');gc();
  check((await iterator.next('chosen')).value === receiver && superWrites[2][0] === receiver && superWrites[2][1] === 'only', 'suspended-super-head-consumes-original-selected-key');
  check((await iterator.next()).done, 'suspended-super-head-completes');
  var abandonedPuts = 0, abandonedTarget = { set chosen(key) { abandonedPuts++; } };
  async function* abandoned() { try { for(abandonedTarget[await (yield 'abandon-key')] in {only:1}) { throw 'entered-abandoned-body'; } } catch(error) { check(error === whole, 'initializer-injected-whole-throw'); yield 'caught-initializer'; } }
  iterator = abandoned(); await iterator.next();
  check((await iterator.throw(whole)).value === 'caught-initializer' && abandonedPuts === 0, 'injected-initializer-throw-retires-without-put');
  check((await iterator.next()).done, 'abandoned-enumerator-does-not-replay');
  var annexEvents = [], annexExcluded = { key: false }, annexTargetCalls = 0;
  var annexView = { set key(next) { annexEvents.push(next); } };
  annexView[Symbol.unscopables] = annexExcluded;
  function annexTarget() { annexTargetCalls++; return null; }
  async function* annexInitializer() { with(annexView) { for(var key=await(yield 'annex-initializer') in (annexTarget(), await(yield 'annex-target'))) { throw 'entered-null-annex-body'; } } }
  iterator = annexInitializer();
  check((await iterator.next()).value === 'annex-initializer' && annexTargetCalls === 0, 'annex-reference-selected-before-initializer-and-target');
  annexExcluded.key = true; gc();
  check((await iterator.next(whole)).value === 'annex-target' && annexEvents.length === 1 && annexEvents[0] === whole && annexTargetCalls === 1, 'annex-initializer-puts-original-reference-before-target');
  check((await iterator.next(null)).done && annexEvents.length === 1 && annexTargetCalls === 1, 'annex-prefix-never-replays-on-null-enumeration');
  async function* rejectedAnnexInitializer() { try { for(var key=await(yield 'rejected-annex-initializer') in annexTarget()) { throw 'entered-rejected-annex-body'; } } catch(error) { check(error === whole, 'annex-initializer-retains-whole-rejection'); yield 'caught-annex-initializer'; } }
  iterator = rejectedAnnexInitializer(); await iterator.next();
  check((await iterator.next(Promise.reject(whole))).value === 'caught-annex-initializer' && annexTargetCalls === 1, 'annex-rejection-prevents-target-and-enumerator');
  check((await iterator.next()).done, 'annex-rejected-reference-retired');
}

async function completions() {
  var read, log = [];
  async function* finishing() { for(const key in await (yield 'finish-head')) { read = function () { return key; }; try { yield 'body'; } finally { log.push(key); await Promise.resolve(0); gc(); yield 'finally'; check(read() === key, 'original-per-key-record-during-finalizer'); } } }
  var iterator = finishing(); await iterator.next(); await iterator.next({ only: 1 });
  var returned = iterator.return(whole), queued = iterator.next();
  check((await returned).value === 'finally', 'queued-return-runs-yielding-finalizer');
  var terminal = await queued; check(terminal.done === true && terminal.value === whole, 'pending-whole-return-survives-yield-and-await');
  check(read() === 'only' && log.join(',') === 'only', 'escaping-original-key-cell-after-cursor-retirement');
  iterator = finishing(); await iterator.next(); await iterator.next({ only: 1 });
  check((await iterator.throw(whole)).value === 'finally', 'injected-throw-enters-original-finalizer');
  try { await iterator.next(); throw 'missing-pending-throw'; } catch(error) { check(error === whole, 'pending-whole-throw-after-finalizer'); }
  var reached = false;
  async function* rejected() { try { for(let key in await (yield 'rejected-head')) { reached = true; } } catch(error) { check(error === whole, 'head-rejection-before-enumerator'); yield 'caught-head'; } }
  iterator = rejected(); await iterator.next();
  check((await iterator.next(Promise.reject(whole))).value === 'caught-head' && reached === false, 'rejected-head-retains-original-cleanup');
  await iterator.next();
  var target = new Proxy({}, { ownKeys: function () { throw whole; } });
  async function* trapped() { try { for(var key in await (yield 'trap-head')) { reached = true; } } catch(error) { check(error === whole, 'enumerator-trap-whole-abrupt'); yield 'caught-trap'; } }
  iterator = trapped(); await iterator.next();
  check((await iterator.next(target)).value === 'caught-trap' && reached === false, 'trap-failure-precedes-eager-key-write');
  await iterator.next();
  var outer = 'outside', seen = [];
  async function* records() { for(const key in await (yield 'records-head')) { with(await (yield 'with-head')) { try { yield outer; } finally { seen.push(outer); await Promise.resolve(0); yield outer; } } } yield outer; }
  iterator = records(); await iterator.next(); check((await iterator.next({ only: 1 })).value === 'with-head', 'nested-with-head');
  check((await iterator.next({ outer: whole })).value === whole, 'nested-original-with-record');
  check((await iterator.return(whole)).value === whole && seen[0] === whole, 'with-and-iteration-remain-during-finalizer');
  terminal = await iterator.next(); check(terminal.done === true && terminal.value === whole && outer === 'outside', 'whole-return-restores-original-record-chain');
}
async function run() { await references(); await completions(); }
run().then(function () { print('mixed-async-generator-for-in-references:ok'); }, function (error) { print(error); throw error; });
