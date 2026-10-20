function check(ok, label) { if (!ok) throw label; }
var calls = 0, rhs = 0;
function target(value) {
  calls++;
  return { [Symbol.toPrimitive]: function () { throw 'call-result-coerced'; } };
}
async function* plain() { target(await (yield 'argument')) = await (yield (rhs++, 'rhs')); }
async function* compound() { target(await (yield 'argument')) += await (yield (rhs++, 'rhs')); }
async function* unreachableRhs() { target(3) = await (yield (rhs++, 'unreachable')); }
async function rejected(promise, expected) {
  try { await promise; } catch (error) {
    check(expected === ReferenceError ? error instanceof ReferenceError : error === expected, 'call-target-error');
    return;
  }
  throw 'missing-call-target-error';
}
async function run() {
  var iterator = plain(), result = await iterator.next();
  check(!result.done && result.value === 'argument' && calls === 0, 'plain-call-argument-first');
  await rejected(iterator.next(1), ReferenceError);
  check(calls === 1 && rhs === 0, 'plain-reference-error-before-rhs');
  iterator = compound(); result = await iterator.next();
  check(!result.done && result.value === 'argument', 'compound-call-argument-first');
  await rejected(iterator.next(2), ReferenceError);
  check(calls === 2 && rhs === 0, 'compound-reference-error-before-rhs-or-coercion');
  await rejected(unreachableRhs().next(), ReferenceError);
  check(calls === 3 && rhs === 0, 'unreachable-rhs-allocates-no-suspension');
  var marker = { abrupt: true };
  iterator = plain(); await iterator.next();
  await rejected(iterator.throw(marker), marker);
  check(calls === 3 && rhs === 0, 'injected-argument-throw-precedes-call');
}
run().then(function () { print('mixed-call-assignment:ok'); }, function (error) { print(error); throw error; });
