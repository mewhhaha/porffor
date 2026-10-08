function check(value, message) { if (!value) throw new Error(message); }
var trace = [], saved, token = { marker: Symbol('plain') };
token.self = token;
Object.defineProperty(globalThis, 'plainGlobal', {
  configurable: true,
  get() { trace.push('get'); throw token; },
  set(value) { trace.push('set'); saved = value; }
});
function* writeGlobal() { return plainGlobal = yield 'global'; }
var iterator = writeGlobal();
check(iterator.next().value === 'global' && trace.length === 0,
      'plain capture does not call the global getter');
gc();
var done = iterator.next(token);
check(done.done && done.value === token && saved === token && trace.join(',') === 'set',
      'Put returns the whole RHS and calls the original setter after suspension');

function rhs(value) { trace.push('rhs'); return value; }
function* tdz() { future = rhs(yield 'tdz'); let future; }
trace = []; iterator = tdz();
check(iterator.next().value === 'tdz' && trace.length === 0, 'TDZ does not precede the RHS');
var caught = null;
try { iterator.next(1); } catch (error) { caught = error; }
check(caught instanceof ReferenceError && trace.join(',') === 'rhs', 'TDZ is checked at PutValue');
function* immutable() { const value = 0; return value = rhs(yield 'const'); }
trace = []; iterator = immutable();
check(iterator.next().value === 'const', 'constant plain write evaluates its RHS');
caught = null;
try { iterator.next(2); } catch (error) { caught = error; }
check(caught instanceof TypeError && trace.join(',') === 'rhs', 'immutable Put follows RHS evaluation');

delete globalThis.originalMissingPlain;
function* unresolvable() { return originalMissingPlain = rhs(yield 'missing'); }
trace = []; iterator = unresolvable();
check(iterator.next().value === 'missing', 'unresolvable target still reaches RHS suspension');
Object.defineProperty(globalThis, 'originalMissingPlain', {
  configurable: true, value: 99, writable: true
});
caught = null; done = null;
try { done = iterator.next(7); } catch (error) { caught = error; }
if (strictPlainFixture) {
  check(caught instanceof ReferenceError && globalThis.originalMissingPlain === 99,
        'the original strict unresolvable Reference is not re-resolved after suspension');
} else {
  check(caught === null && done.done && done.value === 7 && globalThis.originalMissingPlain === 7,
        'sloppy unresolvable Put uses the global object after the RHS');
}
check(trace.join(',') === 'rhs', 'unresolvable failure does not suppress the RHS');
delete globalThis.originalMissingPlain;

var readValue;
function combine(a, b) { return a + b; }
function* multiple() {
  let value = 0; readValue = function () { return value; };
  return value = combine(yield 'one', yield 'two');
}
iterator = multiple();
check(iterator.next().value === 'one' && readValue() === 0, 'capture precedes the complete first operand');
check(iterator.next(2).value === 'two' && readValue() === 0, 'no write between RHS operands');
gc();
check(iterator.next(3).value === 5 && readValue() === 5, 'one final Put consumes the complete RHS');
function* template() { let value = 'old'; return value = `A${yield 'first'}B${yield 'second'}`; }
iterator = template();
check(iterator.next().value === 'first' && iterator.next(2).value === 'second'
      && iterator.next(3).value === 'A2B3', 'template uses the same retained plain Reference');
function* runtime() { let value = 0; eval('var spare = 1;'); return value = yield 'runtime'; }
iterator = runtime();
check(iterator.next().value === 'runtime' && iterator.next(token).value === token,
      'runtime-visible Record is retained without GetValue');
function* iterations() {
  for (let i = 0; i < 2; i++) { let value; value = yield function () { return value; }; }
}
iterator = iterations();
var first = iterator.next().value, second = iterator.next(token).value;
check(first() === token && second() === undefined, 'iteration cells remain distinct during suspension');
check(iterator.next(4).done && first() === token && second() === 4, 'each Put selects its original cell');

function fail() { throw token; }
function* rhsAbrupt() {
  try { future = fail(yield 'rhs-throw'); }
  catch (error) { check(error === token, 'RHS throw precedes TDZ Put'); yield 'handler'; }
  return plainGlobal = yield 'fresh';
  let future;
}
trace = []; iterator = rhsAbrupt();
check(iterator.next().value === 'rhs-throw' && iterator.next(1).value === 'handler',
      'whole RHS abrupt reaches the handler without an assignment failure');
gc();
check(iterator.next().value === 'fresh' && trace.length === 0, 'abandoned Reference is not published');
check(iterator.next(token).value === token && trace.join(',') === 'set', 'fresh capture writes after a yielding handler');
function* finalize() { try { return plainGlobal = yield 'pending'; } finally { yield 'cleanup'; } }
trace = []; iterator = finalize(); iterator.next();
check(iterator.return(token).value === 'cleanup' && trace.length === 0, 'Return abandons the pending plain write');
check(iterator.next().value === token && trace.length === 0, 'whole Return survives a yielding finalizer');
iterator = finalize(); iterator.next();
check(iterator.throw(token).value === 'cleanup', 'Throw enters the finalizer');
caught = null;
try { iterator.next(); } catch (error) { caught = error; }
check(caught === token && caught.self === caught && trace.length === 0,
      'whole Throw survives the finalizer without Put');

var steps = 0;
var delegate = {
  [Symbol.iterator]() { return this; },
  next() { steps++; return steps === 1 ? { done: false, value: 'delegate' } : { done: true, value: token }; },
  return() { return { done: false, value: 'keep' }; }
};
function* delegated() { return plainGlobal = yield* delegate; }
trace = []; iterator = delegated();
check(iterator.next().value === 'delegate' && iterator.return(3).value === 'keep' && trace.length === 0,
      'delegated Return done:false keeps the plain Reference without publishing');
check(iterator.next().value === token && saved === token && trace.join(',') === 'set',
      'delegate completion performs the original retained Put');
print('generator-plain-bindings:ok');
