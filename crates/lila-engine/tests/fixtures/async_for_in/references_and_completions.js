function check(condition, label) { if (!condition) throw label; }
var whole = { marker: 53 }; whole.self = whole;
whole[Symbol.toPrimitive] = function () { throw 'unexpected-conversion'; };
var value = 'global';

async function references() {
  var outer = 'outer', events = [], views = { value: 1 }, excluded = { value: false };
  views[Symbol.unscopables] = excluded;
  var view = new Proxy(views, {
    has: function (object, key) { if (key === 'value') events.push('has'); return Reflect.has(object, key); },
    get: function (object, key, receiver) { if (key === 'value') events.push('get'); return Reflect.get(object, key, receiver); },
    set: function (object, key, next, receiver) { if (key === 'value') events.push('set'); return Reflect.set(object, key, next, receiver); }
  });
  with (view) {
    for (value in await Promise.resolve({ first: 1, second: 2 })) {
      await Promise.resolve(0); gc();
      if (value === 'first') { excluded.value = true; }
    }
  }
  check(views.value === 'first' && value === 'second', 'per-key-original-reference-reselects-after-await');
  check(events.filter(function (event) { return event === 'set'; }).length === 1, 'eager-per-key-put-once');
  excluded.value = false; views.value = 3; events = [];
  with (view) {
    for (const key in await Promise.resolve({ only: 1 })) {
      value += await Promise.resolve(4).then(function (next) {
        excluded.value = true; views.value = 99; gc(); return next;
      });
    }
  }
  check(views.value === 7 && value === 'second', 'body-compound-reference-and-old-value-retained');
  check(events.join(',') === 'has,has,get,has,set', 'compound-has-get-put-before-and-after-await');
  excluded.value = false; events = [];
  with (view) {
    for (const key in await Promise.resolve({ only: 1 })) {
      value = await Promise.resolve(whole).then(function (next) { excluded.value = true; gc(); return next; });
    }
  }
  check(views.value === whole && events.join(',') === 'has,has,set', 'body-write-only-reference-has-no-get');
  excluded.value = false; views.value = null; events = [];
  with (view) {
    for (const key in await Promise.resolve({ only: 1 })) {
      value ??= await Promise.resolve(whole).then(function (next) { excluded.value = true; return next; });
    }
  }
  check(views.value === whole && events.join(',') === 'has,has,get,has,set', 'logical-put-keeps-selected-record');
  excluded.value = false; views.value = 0; events = []; var skipped = 0;
  with (view) {
    for (const key in await Promise.resolve({ only: 1 })) {
      value &&= await { get then() { skipped++; throw whole; } };
    }
  }
  check(skipped === 0 && views.value === 0 && events.join(',') === 'has,has,get', 'logical-skipped-arm-releases-reference-without-put');
  excluded.value = false; events = [];
  with (view) {
    for (const key in await Promise.resolve({ only: 1 })) {
      try { value = await Promise.reject(whole); }
      catch (error) {
        check(error === whole, 'body-rejection-preserves-whole-value');
        await Promise.resolve(0); value = await Promise.resolve(11);
      }
    }
  }
  check(views.value === 11 && events.filter(function (event) { return event === 'set'; }).length === 1, 'abandoned-reference-retired-before-fresh-catch-write');
  var gets = 0, puts = [];
  function selectedTarget() { gets++; return { set key(next) { puts.push(next); } }; }
  for (selectedTarget().key in await Promise.resolve({ aa: 1, bb: 2 })) { await Promise.resolve(0); }
  check(gets === 2 && puts.join(',') === 'aa,bb', 'actual-eager-property-target-before-body');
  var original = async function immutable() {
    for (immutable in await Promise.resolve({ only: 1 })) { await Promise.resolve(0); }
    return immutable;
  };
  check(await original() === original, 'actual-sloppy-ignored-immutable-head-write');
  var strict = async function immutable() {
    'use strict';
    try { for (immutable in await Promise.resolve({ only: 1 })) { throw 'entered-strict-immutable-body'; } }
    catch (error) { check(error instanceof TypeError, 'strict-immutable-head-write-throws'); return whole; }
    throw 'strict-head-did-not-throw';
  };
  check(await strict() === whole, 'strict-head-cleanup-keeps-whole-return');
}

async function cleanup() {
  var value = 'outside', seen = [];
  try {
    for (let key in await Promise.resolve({ first: 1 })) {
      with (await Promise.resolve({ value: whole })) {
        try { await Promise.reject(whole); }
        finally { seen.push(value); await Promise.resolve(0); gc(); seen.push(value); }
      }
    }
  } catch (error) {
    check(error === whole && value === 'outside', 'whole-rejection-restores-iteration-and-with-parents');
    seen.push(value);
  }
  check(seen.length === 3 && seen[0] === whole && seen[1] === whole && seen[2] === 'outside', 'nested-finalizer-before-native-cursor-retirement');
  var read;
  async function returning() {
    for (const key in await Promise.resolve({ first: 1 })) {
      read = function () { return key; };
      try { return whole; }
      finally { await Promise.resolve(0); gc(); check(read() === key, 'same-iteration-record-during-return-finalizer'); }
    }
  }
  check(await returning() === whole && read() === 'first', 'whole-return-and-escaping-key-cell');
  var touched = false;
  try {
    for (var key in await Promise.resolve(new Proxy({}, { ownKeys: function () { throw whole; } }))) { touched = true; }
  } catch (error) { check(error === whole, 'enumerator-trap-whole-abrupt'); }
  check(touched === false, 'trap-failure-precedes-eager-key-write');
}

async function run() { await references(); await cleanup(); }
run().then(function () { print('async-for-in-references:ok'); }, function (error) { print(error); throw error; });
