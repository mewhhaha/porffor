function check(condition, message) { if (!condition) throw new Error(message); }
var calls = 0, argument, reason = {};
function target(value) { calls++; argument = value; return 1; }
function* plain() { target(yield 'argument') = (yield 'unreached-rhs'); }
function* compound() { target(yield 'argument') += (yield 'unreached-rhs'); }
function* update() { target(yield 'argument')++; }
for (var create of [plain, compound, update]) {
  var iterator = create();
  check(iterator.next().value === 'argument', 'original-call-argument');
  var caught = false;
  try { iterator.next(reason); } catch (error) { caught = error instanceof ReferenceError; }
  check(caught && argument === reason, 'ReferenceError-before-RHS-or-numeric-conversion');
  iterator = create(); iterator.next();
  try { iterator.throw(reason); throw 'missing'; } catch (error) { check(error === reason, 'whole-argument-injection'); }
}
check(calls === 3, 'Call-once-and-never-after-injected-throw');
print('generator-call-target:ok');
