function check(condition, label) { if (!condition) throw label; }
var whole = { marker: 83 }; whole.self = whole;
var events = [], x = 'outside';

async function* labelled() {
  outer: for (let index = 0; index < 2; index++) {
    stop: switch (await (yield 'choice')) {
      case 1:
        try { yield whole; continue outer; }
        finally { await Promise.resolve(0); gc(); events.push(index); }
      default:
        try { yield 'default'; break outer; }
        finally { await Promise.resolve(0); gc(); events.push(index); }
    }
  }
  return whole;
}

async function* completing() {
  try {
    switch (await (yield 'discriminant')) {
      case await (yield 'selector'):
        try { yield 'body'; }
        finally { await Promise.resolve(0); gc(); yield whole; }
        break;
      default: throw 'unexpected-default';
    }
  } finally { events.push('outer'); }
}

async function* references(view) {
  with (view) {
    switch (0) {
      case 0:
        x += await (yield 'rhs');
        x ??= await (yield 'forbidden-skipped-rhs');
        var declared = await (yield 'var-rhs');
        yield x;
        break;
    }
  }
  return x;
}

async function* rejectedHead() {
  try { switch (await (yield 'head')) { default: yield 'forbidden-body'; } }
  catch (error) { check(error === whole, 'whole-rejected-head'); yield error; }
}

async function* thrownSelector(view) {
  try { switch (whole) { case view.key: yield 'forbidden-getter-body'; } }
  catch (error) { check(error === whole, 'whole-selector-getter-throw'); yield error; }
}

async function run() {
  var iterator = labelled(), result = await iterator.next(); check(result.value === 'choice', 'labelled-first-discriminant');
  result = await iterator.next(1); check(result.value === whole, 'labelled-first-body');
  result = await iterator.next(); check(result.value === 'choice' && events.join(',') === '0', 'continue-restores-outer-loop-after-finally');
  result = await iterator.next(0); check(result.value === 'default', 'labelled-default-body');
  result = await iterator.next(); check(result.done && result.value === whole && events.join(',') === '0,1', 'break-through-finally-retains-whole-result');
  events = []; iterator = completing(); await iterator.next();
  result = await iterator.return(whole); check(result.done && result.value === whole && events.join(',') === 'outer', 'return-at-discriminant-does-not-enter-case');
  events = []; iterator = completing(); await iterator.next(); result = await iterator.next(1); check(result.value === 'selector', 'selector-suspension');
  result = await iterator.return(whole); check(result.done && result.value === whole && events.join(',') === 'outer', 'return-at-selector-does-not-enter-body');
  events = []; iterator = completing(); await iterator.next(); await iterator.next(1); result = await iterator.next(1); check(result.value === 'body', 'selected-body');
  result = await iterator.return(whole); check(!result.done && result.value === whole, 'return-awaits-and-yields-finally');
  result = await iterator.next(); check(result.done && result.value === whole && events.join(',') === 'outer', 'pending-whole-return-after-finally');
  events = []; iterator = completing(); await iterator.next(); await iterator.next(1); await iterator.next(1);
  result = await iterator.throw(whole); check(!result.done && result.value === whole, 'throw-awaits-and-yields-finally');
  try { await iterator.next(); throw 'missing-whole-throw'; }
  catch (error) { check(error === whole && events.join(',') === 'outer', 'pending-whole-throw-after-finally'); }
  var gets = 0, sets = 0, target = { x: 17, declared: 0 };
  var view = new Proxy(target, {
    get: function (object, key, receiver) { if (key === 'x') gets++; return Reflect.get(object, key, receiver); },
    set: function (object, key, value, receiver) { if (key === 'x') sets++; return Reflect.set(object, key, value, receiver); }
  });
  iterator = references(view); result = await iterator.next();
  check(result.value === 'rhs' && gets === 1 && sets === 0, 'selected-reference-and-old-value-before-rhs');
  target.x = 99; target[Symbol.unscopables] = { x: true }; gc();
  result = await iterator.next(Promise.resolve(5));
  check(result.value === 'var-rhs' && target.x === 22 && gets === 1 && sets === 1, 'original-put-and-skipped-logical-arm');
  result = await iterator.next(Promise.resolve(whole));
  check(result.value === 'outside' && target.declared === whole, 'case-var-initializer-preserves-original-with-reference');
  result = await iterator.next(); check(result.done && result.value === 'outside' && x === 'outside', 'case-and-with-cleanup');
  iterator = rejectedHead(); await iterator.next(); result = await iterator.next(Promise.reject(whole));
  check(result.value === whole && result.value.self === whole, 'head-rejection-retains-whole-completion');
  result = await iterator.next(); check(result.done, 'rejected-head-cleanup');
  var getterView = { get key() { throw whole; } };
  iterator = thrownSelector(getterView); result = await iterator.next();
  check(result.value === whole && result.value.self === whole, 'selector-getter-completion-identity');
  result = await iterator.next(); check(result.done, 'getter-throw-cleanup');
}
run().then(function () { print('mixed-async-generator-switch-completions:ok'); }, function (error) { print(error); throw error; });
