function check(condition, label) { if (!condition) throw label; }
var whole = { marker: 37 }; whole.self = whole;
async function run() {
  var x = 'outside', has = 0, gets = 0, sets = 0, rhs = 0;
  var target = { x: 17 };
  var view = new Proxy(target, {
    has: function (object, key) { if (key === 'x') has++; return key in object; },
    get: function (object, key, receiver) { if (key === 'x') gets++; return Reflect.get(object, key, receiver); },
    set: function (object, key, value, receiver) { if (key === 'x') sets++; return Reflect.set(object, key, value, receiver); }
  });
  with (view) {
    x = await Promise.resolve(whole).then(function (received) {
      target[Symbol.unscopables] = { x: true }; gc(); return received;
    });
  }
  check(target.x === whole && x === 'outside' && has === 2 && gets === 0 && sets === 1, 'plain-reference-selected-before-rhs-without-get');

  target[Symbol.unscopables] = {}; target.x = 17; has = gets = sets = 0;
  with (view) {
    x += await Promise.resolve(5).then(function (received) { target.x = 99; target[Symbol.unscopables] = { x: true }; gc(); return received; });
  }
  check(target.x === 22 && x === 'outside' && has === 3 && gets === 1 && sets === 1, 'compound-original-reference-and-old-value');

  target[Symbol.unscopables] = {}; target.x = undefined; has = gets = sets = 0;
  with (view) {
    x ??= await Promise.resolve(whole).then(function (received) { rhs++; target[Symbol.unscopables] = { x: true }; return received; });
  }
  check(target.x === whole && rhs === 1 && has === 3 && gets === 1 && sets === 1, 'logical-selected-reference-before-await');
  target[Symbol.unscopables] = {}; target.x = 0; has = gets = sets = 0;
  with (view) { x &&= await Promise.resolve(whole).then(function (received) { rhs++; return received; }); }
  check(target.x === 0 && rhs === 1 && has === 2 && gets === 1 && sets === 0, 'logical-skipped-arm-releases-without-put');

  async function declared(view) {
    with (view) { var declaredValue = await Promise.resolve(whole); }
    return declaredValue;
  }
  var declarationView = { declaredValue: 1 };
  check(await declared(declarationView) === undefined && declarationView.declaredValue === whole, 'var-hoisting-versus-original-initializer-reference');

  target[Symbol.unscopables] = {}; target.x = 1;
  with (view) {
    try { x = await Promise.reject(whole); throw 'missing-rhs-rejection'; }
    catch (error) { check(error === whole && target.x === 1, 'abandoned-reference-no-put'); await Promise.resolve(0); }
    x = await Promise.resolve(whole);
  }
  check(target.x === whole, 'fresh-reference-after-caught-rejection');

  var blocked = 0;
  var blockers = { get x() { blocked++; return true; } };
  target[Symbol.unscopables] = blockers; target.x = 7;
  with (view) { x = await Promise.resolve(whole); }
  check(x === whole && target.x === 7 && blocked === 1, 'unscopables-chooses-original-outer-cell');

  var fallback;
  var missing = {};
  with (missing) { fallback = await Promise.resolve(whole).then(function (received) { missing.fallback = 0; return received; }); }
  check(fallback === whole && missing.fallback === 0, 'absent-object-binding-does-not-redirect-put');
}
run().then(function () { print('async-with-references:ok'); }, function (error) { print(error); throw error; });
