function check(condition, label) { if (!condition) throw label; }
var whole = { marker: 61 }; whole.self = whole;
whole[Symbol.toPrimitive] = function () { throw 'whole-conversion'; };
var events = [], read;
async function* complete() {
  for (let index = await Promise.resolve(0); await Promise.resolve(index < 2); index++) {
    read = function () { return index; };
    try {
      var stored = await (yield 'value');
      yield stored;
    } finally {
      events.push(read());
      await Promise.resolve(0); gc();
      yield 'finally:' + read();
    }
  }
  return whole;
}

async function* caught() {
  var value = 1;
  while (await Promise.resolve(true)) {
    try { value += yield 'rhs'; }
    catch (error) {
      check(error === whole, 'caught-injected-whole-throw');
      value = await Promise.resolve(whole);
      yield value;
      break;
    }
  }
  return value;
}

async function* coexist(source) {
  { let outer = whole;
    yield 'before-loop';
    for (let index = 0; index < 1; index++) {
      const reader = function () { return outer; };
      await Promise.resolve(0); gc(); yield reader();
    }
    for await (const value of source) { yield value; }
    await Promise.resolve(0); yield outer;
  }
  return 7;
}

async function* labelled() {
  var visits = 0;
  outer: for (let row = 0; await Promise.resolve(row < 2); row++) {
    for (let column = 0; column < 1; column++) {
      try { visits++; yield row; continue outer; }
      finally { await Promise.resolve(0); yield 'leave:' + row; }
    }
  }
  return visits;
}

async function* foreignBranch(flag) {
  for (const value of [3]) { if (flag) yield value; else yield 0; }
  for (let index = 0; index < 1; index++) { await Promise.resolve(0); yield 9; }
}

async function* foreignCaptured() {
  for await (const value of [1, 2]) {
    { let retained = value;
      var reader = function () { return retained; };
      yield reader(); gc(); yield reader();
    }
  }
}

async function* scopedResource() {
  { let retained = whole;
    var reader = function () { return retained; };
    await using resource = { [Symbol.asyncDispose]: function () { events.push('dispose'); gc(); return Promise.resolve(); } };
    yield reader();
  }
}

async function run() {
  var iterator = complete(), result = await iterator.next();
  check(result.value === 'value' && !result.done, 'first-body-suspend');
  result = await iterator.return(whole);
  check(result.value === 'finally:0' && !result.done && read() === 0, 'return-through-awaited-yielding-finally');
  result = await iterator.next();
  check(result.done && result.value === whole && result.value.self === whole, 'pending-whole-return');
  check(events.join(',') === '0', 'initialization-and-finally-once');
  iterator = complete(); await iterator.next();
  result = await iterator.throw(whole);
  check(result.value === 'finally:0' && !result.done, 'throw-enters-finally');
  try { await iterator.next(); throw 'missing-rejection'; }
  catch (error) { check(error === whole, 'pending-whole-throw'); }
  iterator = caught(); result = await iterator.next();
  check(result.value === 'rhs', 'compound-reference-before-yield');
  result = await iterator.throw(whole);
  check(result.value === whole && !result.done, 'fresh-catch-write-after-abandoned-reference');
  result = await iterator.next();
  check(result.done && result.value === whole, 'implicit-return-await-keeps-whole-value');
  iterator = coexist([1, 2]);
  result = await iterator.next(); check(result.value === 'before-loop', 'captured-block-before-checked-loop');
  result = await iterator.next(); check(result.value === whole, 'checked-outer-block-anchor');
  result = await iterator.next(); check(result.value === 1, 'legacy-for-await-anchor-first');
  result = await iterator.next(); check(result.value === 2, 'legacy-for-await-anchor-second');
  result = await iterator.next(); check(result.value === whole, 'linear-anchor-after-structured-and-foreign-owner');
  result = await iterator.next(); check(result.done && result.value === 7, 'coexisting-protocols-finish');
  iterator = labelled();
  result = await iterator.next(); check(result.value === 0, 'labelled-first');
  result = await iterator.next(); check(result.value === 'leave:0', 'continue-through-finalizer-first');
  result = await iterator.next(); check(result.value === 1, 'labelled-second');
  result = await iterator.next(); check(result.value === 'leave:1', 'continue-through-finalizer-second');
  result = await iterator.next(); check(result.done && result.value === 2, 'outer-continue-target');
  iterator = foreignBranch(true);
  result = await iterator.next(); check(result.value === 3, 'legacy-foreign-yielded-if');
  result = await iterator.next(); check(result.value === 9, 'checked-certificate-relocated-after-legacy-if');
  result = await iterator.next(); check(result.done, 'legacy-if-and-checked-loop-complete');
  iterator = foreignBranch(false);
  result = await iterator.next(); check(result.value === 0, 'legacy-foreign-yielded-else');
  result = await iterator.next(); check(result.value === 9, 'checked-loop-after-legacy-else');
  iterator = foreignCaptured();
  result = await iterator.next(); check(result.value === 1, 'foreign-captured-child-first');
  result = await iterator.next(); check(result.value === 1, 'foreign-captured-child-restored-after-yield');
  result = await iterator.next(); check(result.value === 2, 'foreign-fresh-iteration-record');
  result = await iterator.next(); check(result.value === 2, 'foreign-second-child-restored');
  result = await iterator.next(); check(result.done, 'foreign-captured-child-finish');
  iterator = scopedResource();
  result = await iterator.next(); check(result.value === whole, 'captured-resource-suffix-yield');
  result = await iterator.next(); check(result.done && events[events.length - 1] === 'dispose', 'implicit-disposer-resume-keeps-enclosing-scope');
}
run().then(function () { print('mixed-async-generator-completions:ok'); }, function (error) { print(error); throw error; });
