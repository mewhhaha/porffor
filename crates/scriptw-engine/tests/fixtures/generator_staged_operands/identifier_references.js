function check(value, message) { if (!value) throw new Error(message); }
var token = { big: 18446744073709551616n, symbol: Symbol('reference') };
token.self = token;
var trace = [];
var oldObject = { valueOf() { trace.push('L'); return 3; } };
Object.defineProperty(globalThis, 'suspendedGlobal', {
  configurable: true, get() { trace.push('get'); return oldObject; },
  set(value) { trace.push('set:' + value); }
});
function right(value) { trace.push('make'); return value; }
function* globalCompound() { return suspendedGlobal += right(yield 'global'); }
var globalIterator = globalCompound();
check(globalIterator.next().value === 'global' && trace.join(',') === 'get',
      'global GetValue precedes RHS without early coercion');
gc();
var rhs = { valueOf() { trace.push('R'); return 4; } };
check(globalIterator.next(rhs).value === 7 && trace.join(',') === 'get,make,L,R,set:7',
      'retained whole operands and global selected Reference preserve order');
var capturedValue, changeValue;
function* logical(operation) {
  let value = operation;
  changeValue = function (next) { value = next; };
  capturedValue = function () { return value; };
  return value &&= sum(yield 'one', yield 'two');
}
function sum(a, b) { return a + b; }
var logicalIterator = logical(token);
check(logicalIterator.next().value === 'one', 'selected logical RHS first yield');
changeValue(false);
gc();
check(logicalIterator.next(2).value === 'two' && capturedValue() === false,
      'selected second yield does not re-resolve or write early');
check(logicalIterator.next(5).value === 7 && capturedValue() === 7,
      'selected complete RHS publishes once to retained cell');
function* skipAnd() { const value = false; return value &&= yield 'unreachable'; }
function* skipOr() { const value = token; return value ||= yield 'unreachable'; }
function* skipCoalesce() { const value = 0; return value ??= yield 'unreachable'; }
check(skipAnd().next().value === false && skipOr().next().value === token
      && skipCoalesce().next().value === 0, 'all skipped logical assignments avoid const PutValue');
function* selectImmutable() { const value = true; return value &&= yield 'immutable'; }
var immutable = selectImmutable();
check(immutable.next().value === 'immutable', 'selected const RHS runs before immutable failure');
var caught = null;
try { immutable.next(token); } catch (error) { caught = error; }
check(caught instanceof TypeError, 'selected logical const PutValue throws after RHS');
function* noBinding() { return neverCreatedReference += yield 'unreachable'; }
caught = null;
try { noBinding().next(); } catch (error) { caught = error; }
check(caught instanceof ReferenceError, 'unresolvable GetValue throws before RHS');
Object.defineProperty(globalThis, 'deletedReference', { configurable: true, value: 6, writable: true });
function* deletedGlobal() { return deletedReference += yield 'delete'; }
var deletedIterator = deletedGlobal();
check(deletedIterator.next().value === 'delete', 'selected global before deletion');
delete globalThis.deletedReference;
caught = null;
var deletedResult;
try { deletedResult = deletedIterator.next(2); } catch (error) { caught = error; }
if (strictReferenceFixture) {
  check(caught instanceof ReferenceError && !('deletedReference' in globalThis),
        'strict selected Object record rechecks its original property at PutValue');
} else {
  check(caught === null && deletedResult.value === 8 && globalThis.deletedReference === 8,
        'sloppy selected original Object record recreates deleted property');
}
function* abrupt() { let value = 3; capturedValue = function () { return value; }; return value += yield 'abrupt'; }
var returning = abrupt(); returning.next();
check(returning.return(token).value === token && capturedValue() === 3,
      'Return skips retained Reference publication');
var throwing = abrupt(); throwing.next(); caught = null;
try { throwing.throw(token); } catch (error) { caught = error; }
check(caught === token && capturedValue() === 3, 'whole Throw skips publication');
function* runtimeReference() { let value = 2; eval('var spare = 1;'); return value += yield 'runtime'; }
var runtime = runtimeReference();
check(runtime.next().value === 'runtime' && runtime.next(5).value === 7,
      'runtime-visible selection retains real named record');
function throwRhs(value) { throw token; }
function* caughtRhs() {
  let value = 1;
  capturedValue = function () { return value; };
  try { value += throwRhs(yield 'caught-rhs'); }
  catch (error) { check(error === token, 'caught whole RHS throw'); yield 'handler'; }
  return value += yield 'fresh';
}
var caughtIterator = caughtRhs();
check(caughtIterator.next().value === 'caught-rhs', 'RHS before abrupt transfer');
check(caughtIterator.next(0).value === 'handler' && capturedValue() === 1,
      'abandoned Reference does not publish while catch suspends');
check(caughtIterator.next().value === 'fresh' && caughtIterator.next(2).value === 3,
      'new assignment after resumed catch gets its own Reference');
function* finallyRhs() {
  let value = 1;
  capturedValue = function () { return value; };
  try { return value += yield 'pending'; }
  finally { yield 'finalizer'; }
}
var finallyReturning = finallyRhs(); finallyReturning.next();
check(finallyReturning.return(token).value === 'finalizer' && capturedValue() === 1,
      'committed Return abandons Reference before yielding finalizer');
check(finallyReturning.next().value === token && capturedValue() === 1,
      'pending whole Return survives finalizer without assignment');
var finallyThrowing = finallyRhs(); finallyThrowing.next();
check(finallyThrowing.throw(token).value === 'finalizer', 'Throw runs yielding finalizer');
caught = null;
try { finallyThrowing.next(); } catch (error) { caught = error; }
check(caught === token && capturedValue() === 1,
      'pending whole Throw survives finalizer without assignment');
var delegatedSteps = 0;
var delegated = {
  [Symbol.iterator]() { return this; },
  next() {
    delegatedSteps++;
    if (delegatedSteps === 1) return { done: false, value: 'delegated' };
    return { done: true, value: 5 };
  },
  return(value) { return { done: false, value: 'keep-delegating' }; }
};
function* delegatedRhs() {
  let value = 1;
  capturedValue = function () { return value; };
  return value += yield* delegated;
}
var delegating = delegatedRhs();
check(delegating.next().value === 'delegated', 'delegated RHS first yield');
var kept = delegating.return(token);
check(!kept.done && kept.value === 'keep-delegating' && capturedValue() === 1,
      'delegated return(done:false) keeps the live Reference');
check(delegating.next().value === 6 && capturedValue() === 6,
      'later delegated normal completion consumes retained Reference');
print('generator-identifier-reference:ok');
