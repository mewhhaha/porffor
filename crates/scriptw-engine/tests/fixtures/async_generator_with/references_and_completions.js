function check(condition, label) { if (!condition) throw label; }
var whole = { marker: 73 }; whole.self = whole;
var outer = 'outside', events = [];

async function* plain(view) { with (view) { x = await (yield 'plain-rhs'); } return x; }
async function* compound(view) { with (view) { x += await (yield 'compound-rhs'); } return x; }
async function* logical(view) { with (view) { x ??= await (yield 'logical-rhs'); } return x; }
async function* skipped(view) { with (view) { x &&= await (yield 'forbidden-rhs'); } return x; }
async function* declared(view) { with (view) { var declaredValue = await (yield 'declared-rhs'); } return declaredValue; }

async function* caught(view) {
  with (view) {
    try { x = await (yield 'abandoned-rhs'); }
    catch (error) {
      check(error === whole, 'caught-whole-injection');
      await Promise.resolve(0); gc();
      x = await (yield 'fresh-rhs');
    }
    yield x;
  }
}

async function* completing(view) {
  try {
    with (view) {
      try { yield 'body'; }
      finally { events.push(outer); await Promise.resolve(0); gc(); yield outer; }
    }
  } finally { events.push(outer); }
}

async function run() {
  var x = 'outside', gets = 0, sets = 0;
  var target = { x: 17 };
  var view = new Proxy(target, {
    get: function (object, key, receiver) { if (key === 'x') gets++; return Reflect.get(object, key, receiver); },
    set: function (object, key, value, receiver) { if (key === 'x') sets++; return Reflect.set(object, key, value, receiver); }
  });
  // The outer binding is global to the generators; the driver's local x is
  // separately checked below so a selected ObjectER never writes it.
  globalThis.x = 'global-outside';
  var iterator = plain(view), result = await iterator.next();
  check(result.value === 'plain-rhs' && gets === 0 && sets === 0, 'write-only-original-reference-before-rhs');
  target[Symbol.unscopables] = { x: true }; gc();
  result = await iterator.next(Promise.resolve(whole));
  check(result.done && target.x === whole && globalThis.x === 'global-outside' && x === 'outside' && gets === 0 && sets === 1, 'plain-put-retains-object-record');
  target[Symbol.unscopables] = {}; target.x = 17; gets = sets = 0;
  iterator = compound(view); result = await iterator.next();
  check(result.value === 'compound-rhs' && gets === 1 && sets === 0, 'compound-gets-before-rhs');
  target.x = 99; target[Symbol.unscopables] = { x: true }; gc();
  result = await iterator.next(Promise.resolve(5));
  check(result.done && target.x === 22 && globalThis.x === 'global-outside' && gets === 1 && sets === 1, 'compound-keeps-original-old-value-and-put');
  target[Symbol.unscopables] = {}; target.x = undefined; gets = sets = 0;
  iterator = logical(view); result = await iterator.next(); check(result.value === 'logical-rhs', 'logical-selected');
  target[Symbol.unscopables] = { x: true }; gc(); result = await iterator.next(Promise.resolve(whole));
  check(result.done && target.x === whole && gets === 1 && sets === 1, 'logical-selected-original-reference');
  target[Symbol.unscopables] = {}; target.x = 0; gets = sets = 0;
  iterator = skipped(view); result = await iterator.next();
  check(result.done && target.x === 0 && gets === 1 && sets === 0, 'logical-skipped-arm-releases-without-rhs');
  var declarationView = { declaredValue: 1 };
  iterator = declared(declarationView); result = await iterator.next(); check(result.value === 'declared-rhs', 'var-original-initializer-reference');
  result = await iterator.next(Promise.resolve(whole));
  check(result.done && result.value === undefined && declarationView.declaredValue === whole, 'var-hoisting-is-separate-from-with-put');
  target[Symbol.unscopables] = {}; target.x = 1;
  iterator = caught(view); result = await iterator.next(); check(result.value === 'abandoned-rhs', 'captured-reference-before-throw');
  result = await iterator.throw(whole); check(result.value === 'fresh-rhs' && target.x === 1, 'abrupt-reference-retired-before-fresh-capture');
  result = await iterator.next(Promise.resolve(whole)); check(result.value === whole && target.x === whole, 'fresh-reference-after-caught-throw');
  result = await iterator.next(); check(result.done, 'caught-cleanup');
  var blockers = { x: true }; target[Symbol.unscopables] = blockers; target.x = 7;
  iterator = plain(view); result = await iterator.next();
  blockers.x = false; gc(); result = await iterator.next(Promise.resolve(whole));
  check(result.done && globalThis.x === whole && target.x === 7, 'unscopables-retains-original-global-reference');
  var completionView = { outer: whole };
  iterator = completing(completionView); result = await iterator.next(); check(result.value === 'body', 'return-body');
  result = await iterator.return(whole); check(!result.done && result.value === whole, 'pending-return-awaits-and-yields-under-with');
  result = await iterator.next(); check(result.done && result.value === whole && result.value.self === whole, 'whole-pending-return');
  check(events[0] === whole && events[1] === 'outside', 'return-restores-with-before-outer-finally');
  events = []; iterator = completing(completionView); await iterator.next();
  result = await iterator.throw(whole); check(!result.done && result.value === whole, 'pending-throw-yielding-finally');
  try { await iterator.next(); throw 'missing-whole-throw'; }
  catch (error) { check(error === whole && events[0] === whole && events[1] === 'outside', 'whole-throw-restores-record-before-outer-finally'); }
}
run().then(function () { print('mixed-async-generator-with-references:ok'); }, function (error) { print(error); throw error; });
