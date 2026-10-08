function assert(value, message) { if (!value) throw new Error(message); }
function step(iterator, value, expected, message) {
  var result = iterator.next(value);
  assert(!result.done && result.value === expected, message);
}
function done(iterator, value, expected, message) {
  var result = iterator.next(value);
  assert(result.done && result.value === expected, message);
}

var raw = { identity: 1 };
raw.self = raw;
raw[Symbol.toPrimitive] = function () { throw new Error('selector coerced identity'); };
var trace = [];
var selector = { get chosen() { trace.push('get'); return raw; } };
function poisonSelector() { throw new Error('fallthrough evaluated selector'); }
function* choose() {
  switch ((trace.push('d1'), yield 'd1', trace.push('d2'), yield 'd2')) {
    case (yield 's1', yield 's2', selector.chosen):
      trace.push('first'); yield 'b1';
    default:
      trace.push('default'); yield 'bd';
    case poisonSelector():
      trace.push('last'); yield 'b3'; break;
  }
  return raw;
}
var choice = choose();
step(choice, undefined, 'd1', 'first full discriminant segment');
gc();
step(choice, undefined, 'd2', 'second full discriminant segment');
step(choice, raw, 's1', 'first selected test segment');
gc();
step(choice, undefined, 's2', 'second selected test segment');
step(choice, undefined, 'b1', 'whole raw identity strictly matches');
gc();
step(choice, undefined, 'bd', 'fallthrough executes default body');
step(choice, undefined, 'b3', 'fallthrough skips later poison selector');
done(choice, undefined, raw, 'discriminant referent retains identity');
assert(trace.join(',') === 'd1,d2,get,first,default,last', 'no phase replay or coercion');

function* defaults() {
  switch (0) {
    case yield 'before': yield 'before-body';
    default: yield 'default-body';
    case yield 'after': yield 'after-body'; break;
  }
  return 'default-done';
}
var missed = defaults();
step(missed, undefined, 'before', 'selector preceding default');
step(missed, 9, 'after', 'miss still tests selector following default');
step(missed, 8, 'default-body', 'default chosen after all tests miss');
step(missed, undefined, 'after-body', 'default normal fallthrough');
done(missed, undefined, 'default-done', 'default completion');
var after = defaults();
step(after, undefined, 'before', 'after match first selector');
step(after, 1, 'after', 'after match second selector');
step(after, 0, 'after-body', 'after-default match skips default body');
done(after, undefined, 'default-done', 'after-default matched completion');
var before = defaults();
step(before, undefined, 'before', 'before match selector');
step(before, 0, 'before-body', 'before-default direct match');
step(before, undefined, 'default-body', 'matched fallthrough reaches default');
step(before, undefined, 'after-body', 'matched fallthrough skips after selector');
done(before, undefined, 'default-done', 'before-default matched completion');

function* sharedCells() {
  var reader;
  switch (yield 'shared-choose') {
    case 1:
      let shared = yield 'shared-init';
      reader = function () { return shared; };
      yield reader;
    default:
      shared++;
      yield reader;
      break;
  }
  return reader;
}
var cells = sharedCells();
step(cells, undefined, 'shared-choose', 'shared CaseBlock selection');
step(cells, 1, 'shared-init', 'shared lexical initialization');
var firstReader = cells.next(41).value;
gc();
assert(firstReader() === 41, 'first retained shared cell');
var secondReader = cells.next().value;
assert(secondReader === firstReader && secondReader() === 42, 'fallthrough retains one CaseBlock record');
gc();
done(cells, undefined, firstReader, 'escaped reader survives Switch exit');
assert(firstReader() === 42, 'escaped shared cell remains live');

var slot = 'outer-slot';
function* discriminantScope() {
  switch (yield slot) {
    case 1: let slot = 7; yield slot; break;
  }
}
var beforeScope = discriminantScope();
step(beforeScope, undefined, 'outer-slot', 'discriminant precedes CaseBlock instantiation');
step(beforeScope, 1, 7, 'actual inner lexical slot initialized');
done(beforeScope, undefined, undefined, 'scope completion');
function* selectorTdz() {
  switch (yield 'tdz') {
    case slot: let slot = 7; yield 'unreachable';
  }
}
var tdz = selectorTdz();
step(tdz, undefined, 'tdz', 'TDZ discriminator suspension');
var tdzThrown = false;
try { tdz.next(7); } catch (error) { tdzThrown = error instanceof ReferenceError; }
assert(tdzThrown, 'all CaseBlock declarations precede selector evaluation');
function* hoistedCaseFunction() {
  switch (yield 'hoisted') {
    case f(): yield 'hoisted-body'; break;
    default: function f() { return 1; }
  }
}
var hoisted = hoistedCaseFunction();
step(hoisted, undefined, 'hoisted', 'hoisted discriminator');
step(hoisted, 1, 'hoisted-body', 'case functions instantiated before any test');
done(hoisted, undefined, undefined, 'hoisted completion');

function* nestedEmpty() {
  switch (yield 'nested-choose') {
    case 0: {
      6;
      const fixed = yield 'const';
      if (true) var variable = yield 'var';
      try { let local = yield 'let'; yield fixed + variable + local; }
      finally { yield 'nested-finally'; }
      for (var i = 0; i < 1; i++) { let loopLocal = yield 'loop'; yield loopLocal; }
      break;
    }
  }
  return 'nested-done';
}
var nested = nestedEmpty();
step(nested, undefined, 'nested-choose', 'nested selection');
step(nested, 0, 'const', 'nested const group');
gc();
step(nested, 2, 'var', 'nested If var group');
step(nested, 3, 'let', 'nested Try let group');
gc();
step(nested, 4, 9, 'nested actual expression completion');
step(nested, 17, 'nested-finally', 'bare Yield resumes actual statement value');
step(nested, undefined, 'loop', 'nested classic loop lexical group');
step(nested, 12, 12, 'nested loop received cell');
done(nested, undefined, 'nested-done', 'nested Empty groups resume exact region');

function* labels() {
  outer: for (let i = 0; i < 3; i++) {
    namedSwitch: switch (yield ('choose:' + i)) {
      case 0:
        try { yield 'continue'; continue outer; }
        finally { yield 'continue-finally'; }
      default:
        inner: switch (yield 'inner') {
          case 1: yield 'inner-body'; break inner;
          default: throw new Error('wrong nested case');
        }
        try { yield 'break'; break outer; }
        finally { yield 'break-finally'; }
    }
  }
  return 'labels-done';
}
var labelled = labels();
step(labelled, undefined, 'choose:0', 'outer loop first selection');
step(labelled, 0, 'continue', 'selected continue body');
step(labelled, undefined, 'continue-finally', 'outer Continue enters finalizer');
gc();
step(labelled, undefined, 'choose:1', 'resumed outer Continue reaches update');
step(labelled, 1, 'inner', 'nested Switch discrimination');
step(labelled, 1, 'inner-body', 'nested selected body');
step(labelled, undefined, 'break', 'nested labelled Break restores outer Switch');
step(labelled, undefined, 'break-finally', 'outer Break enters finalizer');
done(labelled, undefined, 'labels-done', 'resumed outer Break exits loop');

var whole = { kind: 'whole-completion' };
whole.self = whole;
var throwingSelector = { get value() { throw whole; } };
function* caughtSelector() {
  try {
    switch (yield 'trap') { case throwingSelector.value: yield 'unreachable'; }
  } catch (error) { yield error; yield 'caught'; return error; }
}
var caught = caughtSelector();
step(caught, undefined, 'trap', 'getter trap discriminant');
step(caught, 1, whole, 'arbitrary selector getter Throw');
gc();
step(caught, undefined, 'caught', 'catch region resumes once');
done(caught, undefined, whole, 'caught whole cyclic identity');
function* abrupt(point) {
  try {
    switch (point === 0 ? yield 'disc' : 0) {
      case point === 1 ? yield 'select' : 0:
        if (point === 2) yield 'body';
        break;
    }
  } finally { gc(); yield 'cleanup'; }
  return 'unreachable-return';
}
for (var point = 0; point < 3; point++) {
  var expected = point === 0 ? 'disc' : point === 1 ? 'select' : 'body';
  var returning = abrupt(point);
  step(returning, undefined, expected, 'injected Return phase');
  var pendingReturn = returning.return(whole);
  assert(!pendingReturn.done && pendingReturn.value === 'cleanup', 'Return crosses yielding finalizer');
  gc();
  done(returning, undefined, whole, 'pending whole Return preserved');
  var throwing = abrupt(point);
  step(throwing, undefined, expected, 'injected Throw phase');
  var pendingThrow = throwing.throw(whole);
  assert(!pendingThrow.done && pendingThrow.value === 'cleanup', 'Throw crosses yielding finalizer');
  gc();
  var wholeThrown = false;
  try { throwing.next(); } catch (error) { wholeThrown = error === whole; }
  assert(wholeThrown, 'pending whole Throw preserved');
}
print('generator-switch-regions:ok');
true;
