function check(condition, label) { if (!condition) throw label; }
var value = 'outside';
var whole = { marker: 71 }; whole.self = whole;
whole[Symbol.toPrimitive] = function () { throw 'unexpected-whole-conversion'; };
var events = [], reader;

async function* environments(view, other) {
  with (await (yield 'head')) {
    let local = whole;
    reader = function () { return [value, local]; };
    yield reader();
    await Promise.resolve(0); gc();
    check(reader()[0] === value && reader()[1] === whole, 'original-object-and-nested-block-cells');
    if (await (yield 'choice')) {
      with (await Promise.resolve(other)) { yield value; }
    } else { throw 'wrong-branch'; }
    for (let index = await Promise.resolve(0); await Promise.resolve(index < 2); index++) {
      try { yield value; }
      finally { await Promise.resolve(0); gc(); events.push(index); }
    }
    stop: with (view) { await Promise.resolve(0); break stop; }
    yield value;
  }
  check(value === 'outside', 'normal-with-exit-restores-outer-record');
  return reader;
}

async function* eager(view) {
  if (true) {
    with (view) { return function () { return value; }; }
  }
  throw 'unreachable-eager-arm';
}

async function* rejectHead() {
  with (await (yield 'null-head')) { throw 'entered-null-with'; }
}

async function* wholeHead(view) {
  with (await Promise.resolve(view).then(function (received) { gc(); return received; })) {
    yield value;
  }
}

// A noniteration With label accepts only its named Break. An outer
// iteration label remains a Continue target through the original cleanup.
async function* labelledExits(view) {
  var trace = [];
  outer: for (let index = 0; index < 2; index++) {
    stopped: with (view) {
      try {
        await Promise.resolve(0);
        if (index === 0) continue outer;
        break stopped;
      } finally {
        await Promise.resolve(0); gc(); trace.push(index + ':' + value.marker);
      }
    }
    check(value === 'outside', 'labelled-break-leaves-with-before-outer-body');
    yield index;
  }
  check(value === 'outside', 'labelled-continue-leaves-with-before-next-iteration');
  return trace;
}

async function* nestedLabels(view) {
  first: second: with (view) { await Promise.resolve(0); break second; }
  yield value;
  first: second: with (view) { await Promise.resolve(0); break first; }
  await Promise.resolve(0); gc();
  return value;
}

async function run() {
  var view = { value: whole }, other = { value: 42 };
  var iterator = environments(view, other), result = await iterator.next();
  check(result.value === 'head' && !result.done, 'head-suspends-before-record-entry');
  result = await iterator.next(Promise.resolve(view));
  check(result.value[0] === whole && result.value[1] === whole, 'head-await-and-original-closure');
  view.value = other; gc();
  result = await iterator.next(); check(result.value === 'choice', 'body-resumes-original-with-chain');
  result = await iterator.next(true); check(result.value === 42, 'nested-complete-with-record');
  result = await iterator.next(); check(result.value === other, 'mixed-loop-first');
  result = await iterator.next(); check(result.value === other, 'mixed-loop-second');
  result = await iterator.next(); check(result.value === other, 'labelled-with-restores-enclosing-record');
  result = await iterator.next();
  check(result.done && result.value === reader && reader()[0] === other && reader()[1] === whole, 'escaping-closure-retains-original-record');
  check(events.join(',') === '0,1', 'nested-loop-finalizers-once');
  iterator = eager({ value: whole }); result = await iterator.next();
  check(result.done && result.value() === whole, 'eager-with-still-owns-phases-and-hidden-cell');
  iterator = rejectHead(); result = await iterator.next(); check(result.value === 'null-head', 'null-head-yield');
  try { await iterator.next(null); throw 'missing-head-type-error'; }
  catch (error) { check(error instanceof TypeError && value === 'outside', 'to-object-before-environment-entry'); }
  iterator = wholeHead({ value: whole }); result = await iterator.next();
  check(result.value === whole && result.value.self === whole, 'head-await-retains-whole-value-through-gc');
  result = await iterator.next(); check(result.done, 'whole-head-cleanup');
  iterator = labelledExits({ value: whole });
  result = await iterator.next();
  check(!result.done && result.value === 1, 'outer-continue-reaches-second-iteration');
  result = await iterator.next();
  check(result.done && result.value.join(',') === '0:71,1:71', 'labelled-with-finalizers-once-in-original-record');
  iterator = nestedLabels({ value: whole });
  result = await iterator.next();
  check(!result.done && result.value === 'outside', 'inner-with-label-continues-following-yield');
  result = await iterator.next();
  check(result.done && result.value === 'outside', 'outer-with-label-continues-following-await');
}
run().then(function () { print('mixed-async-generator-with-environments:ok'); }, function (error) { print(error); throw error; });
