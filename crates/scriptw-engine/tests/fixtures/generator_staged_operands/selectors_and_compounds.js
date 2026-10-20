function assert(value, message) { if (!value) throw new Error(message); }
function* andValue() { return (yield 'left') && (yield 'right'); }
var skippedAnd = andValue();
assert(skippedAnd.next().value === 'left', 'left yields first');
var andDone = skippedAnd.next(false);
assert(andDone.done && andDone.value === false, 'false left skips right and retains value');
var selectedAnd = andValue();
selectedAnd.next();
assert(selectedAnd.next(true).value === 'right', 'truthy left selects right');
var token = { value: 9007199254740993n };
token.self = token;
gc();
assert(selectedAnd.next(token).value === token, 'selected value identity');
function* orValue() { return (yield 'left') || (yield 'right'); }
var skippedOr = orValue();
skippedOr.next();
assert(skippedOr.next(token).value === token, 'truthy object skips RHS without coercion');
function* coalesceValue() { return (yield 'left') ?? (yield 'right'); }
var selectedCoalesce = coalesceValue();
selectedCoalesce.next();
assert(selectedCoalesce.next(undefined).value === 'right', 'nullish left selects RHS');
assert(selectedCoalesce.next(token).value === token, 'nullish selected identity');
var skippedCoalesce = coalesceValue();
skippedCoalesce.next();
var coalesceDone = skippedCoalesce.next(false);
assert(coalesceDone.done && coalesceDone.value === false, 'non-nullish false is retained');
function* nestedValue() { return ((yield 'a') && (yield 'b')) || (yield 'c'); }
var nested = nestedValue();
assert(nested.next().value === 'a', 'nested first selector');
assert(nested.next(true).value === 'b', 'nested selected left branch');
assert(nested.next(false).value === 'c', 'completed nested value selects outer RHS');
assert(nested.next(token).value === token, 'nested phi identity');

var change, read;
function* compound() {
  let value = 3;
  change = function (next) { value = next; };
  read = function () { return value; };
  return value += yield 'operand';
}
var compoundIterator = compound();
assert(compoundIterator.next().value === 'operand', 'compound suspends after GetValue');
change(100);
gc();
var compoundDone = compoundIterator.next(4);
assert(compoundDone.done && compoundDone.value === 7 && read() === 7,
       'saved old value and original captured cell survive caller mutation');
var returning = compound();
returning.next();
assert(returning.return(token).value === token && read() === 3, 'Return skips compound PutValue');
var throwing = compound();
throwing.next();
var caught = null;
try { throwing.throw(token); } catch (error) { caught = error; }
assert(caught === token && read() === 3, 'Throw skips PutValue and retains whole identity');

var order = [], oldObject = { valueOf() { order.push('L'); return 4; } };
function makeRight(value) { order.push('make'); return value; }
function* coercion() {
  let value = oldObject;
  change = function (next) { value = next; };
  return value += makeRight(yield 'operand');
}
var coercing = coercion();
coercing.next();
change(1000);
gc();
var rightObject = { valueOf() { order.push('R'); return 5; } };
assert(coercing.next(rightObject).value === 9 && order.join(',') === 'make,L,R',
       'whole old operand is coerced after RHS evaluation and before RHS coercion');
function* immutable() { const value = { valueOf() { throw token; } }; return value += yield 1; }
var immutableIterator = immutable();
immutableIterator.next();
caught = null;
try { immutableIterator.next(2); } catch (error) { caught = error; }
assert(caught === token, 'operator throw precedes immutable PutValue');
function* tdz() { value += yield 'unreachable'; let value; }
caught = null;
try { tdz().next(); } catch (error) { caught = error; }
assert(caught instanceof ReferenceError, 'GetValue TDZ throws before RHS suspension');
function* bits() { let value = 6n; return value ^= yield 'bits'; }
var bitsIterator = bits();
assert(bitsIterator.next().value === 'bits' && bitsIterator.next(3n).value === 5n,
       'closed bitwise operation keeps BigInt semantics');

var cells = [];
function* loop() {
  for (let i = 0; (yield ('test:' + i)) && i < 2; i += yield ('step:' + i)) {
    cells.push(function () { return i; });
    yield ('body:' + i);
  }
  return cells[0]() + ',' + cells[1]();
}
var looping = loop();
assert(looping.next().value === 'test:0', 'logical test prefix');
assert(looping.next(true).value === 'body:0', 'first body');
assert(looping.next().value === 'step:0', 'compound update prefix');
gc();
assert(looping.next(1).value === 'test:1', 'compound update precedes next selector');
assert(looping.next(true).value === 'body:1', 'second body');
assert(looping.next().value === 'step:1', 'second compound update');
assert(looping.next(1).value === 'test:2', 'second update resumes');
var loopDone = looping.next(true);
assert(loopDone.done && loopDone.value === '0,1', 'actual fresh iteration cells survive both new owners');
print('generator-staged-operands:ok');
