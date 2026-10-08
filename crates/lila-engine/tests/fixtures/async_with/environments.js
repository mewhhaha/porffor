function check(condition, label) { if (!condition) throw label; }
var whole = { marker: 31 }; whole.self = whole;
whole[Symbol.toPrimitive] = function () { throw 'unexpected-conversion'; };
var value = 'global';

async function captured(view) {
  var reader;
  with (await Promise.resolve(view).then(function (object) { gc(); return object; })) {
    reader = function () { return value; };
    await Promise.resolve(0);
    check(reader() === value, 'same-object-record-after-await');
  }
  gc(); return reader;
}

async function branches(view) {
  var result;
  with (await Promise.resolve(view)) {
    if (await Promise.resolve(true)) {
      const [received = await Promise.resolve(whole)] = [];
      result = received;
    } else { result = await Promise.resolve(0); }
    switch (await Promise.resolve(2)) {
      case await Promise.resolve(1): throw 'wrong-selector';
      default: throw 'wrong-default';
      case await Promise.resolve(2):
        with (await Promise.resolve({ value: whole })) {
          check(value === whole, 'nested-with-selection');
          await Promise.resolve(0); break;
        }
    }
    outside: {
      with ({ value: whole }) {
        await Promise.resolve(0);
        check(value === whole, 'label-body-record');
        break outside;
      }
    }
  }
  check(value === 'global', 'labelled-break-restores-environment');
  return result;
}

async function rejected(view, events) {
  var value = 'outside';
  try {
    with (await Promise.resolve(view)) {
      try { await Promise.reject(whole); }
      finally {
        events.push(value);
        await Promise.resolve(0);
        gc(); events.push(value);
      }
    }
  } catch (error) {
    check(error === whole && value === 'outside', 'whole-rejection-and-outer-record');
    events.push(value);
  }
  return value;
}

async function returning(view, events) {
  with (view) {
    try { return whole; }
    finally { events.push(value); await Promise.resolve(0); events.push(value); }
  }
}

async function eager(view) {
  if (true) { with (view) { return function () { return value; }; } }
  throw 'unreachable-eager-arm';
}

async function run() {
  var firstView = { value: whole }, secondView = { value: 42 };
  var first = captured(firstView), second = captured(secondView);
  var firstReader = await first, secondReader = await second;
  firstView.value = secondView; gc();
  check(firstReader() === secondView && secondReader() === 42, 'independent-original-records-and-escaping-closures');
  check(await branches({ value: whole }) === whole, 'complete-branches-and-array-default');
  var events = [];
  check(await rejected({ value: whole }, events) === 'outside', 'rejected-return-value');
  check(events.length === 3 && events[0] === whole && events[1] === whole && events[2] === 'outside', 'awaited-finally-before-with-cleanup');
  events = [];
  check(await returning({ value: whole }, events) === whole && events[0] === whole && events[1] === whole, 'whole-return-survives-awaiting-finalizer');
  check((await eager({ value: whole }))() === whole, 'eager-with-has-real-phases');
  var seen = 0, forbidden = { get then() { seen++; throw whole; } };
  with ((null?.[await forbidden], { value: whole })) {
    var skipped = null?.[await forbidden]();
    check(skipped === undefined && seen === 0, 'checked-known-nullish-tails-keep-skipped-awaits');
  }
  try { with (await Promise.resolve(null)) { throw 'entered-null-with'; } }
  catch (error) { check(error instanceof TypeError && value === 'global', 'head-to-object-fails-before-record-entry'); }
  var count = 0;
  while (await Promise.resolve(count++ < 1)) { with ({ value: whole }) { check(value === whole, 'original-eager-loop-with'); } }
  var keys = [];
  for (var key in { first: 1, second: 2 }) {
    with ({ value: whole }) { check(value === whole, 'original-eager-for-in-with'); }
    keys.push(key);
  }
  check(keys.join(',') === 'first,second', 'plain-async-for-in-retains-original-enumerator');
}
run().then(function () { print('async-with-environments:ok'); }, function (error) { print(error); throw error; });
