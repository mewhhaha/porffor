function assert(value, message) { if (!value) throw new Error(message); }
function* labels() {
  outer: for (let i = 0; i < 3; i++) {
    try {
      if (i === 0) {
        yield 'body:0';
        continue outer;
      }
      yield ('body:' + i);
      break outer;
    } finally {
      yield ('finally:' + i);
    }
  }
  return 'done';
}
var labelled = labels();
assert(labelled.next().value === 'body:0', 'first labelled body');
assert(labelled.next().value === 'finally:0', 'Continue enters finalizer');
assert(labelled.next().value === 'body:1', 'resumed Continue reaches update');
assert(labelled.next().value === 'finally:1', 'Break enters finalizer');
var labelledDone = labelled.next();
assert(labelledDone.done && labelledDone.value === 'done', 'resumed Break exits exact loop');
print('labels:true');

function* nested() {
  var i = 0, j;
  outer: while (i < 2) {
    for (j = 0; j < 2; j++) {
      try {
        yield ('nested:' + i + ':' + j);
        if (i === 0) { i++; continue outer; }
        if (j === 0) continue;
        break outer;
      } finally {
        yield ('nested-finally:' + i + ':' + j);
      }
    }
  }
  return i;
}
var nestedIterator = nested();
assert(nestedIterator.next().value === 'nested:0:0', 'first nested body');
assert(nestedIterator.next().value === 'nested-finally:1:0', 'outer Continue crosses inner finalizer');
assert(nestedIterator.next().value === 'nested:1:0', 'outer Continue resumes enclosing head');
assert(nestedIterator.next().value === 'nested-finally:1:0', 'inner Continue crosses its finalizer');
assert(nestedIterator.next().value === 'nested:1:1', 'inner Continue resumes its own update');
assert(nestedIterator.next().value === 'nested-finally:1:1', 'outer Break crosses inner finalizer');
var nestedDone = nestedIterator.next();
assert(nestedDone.done && nestedDone.value === 1, 'outer Break exits both actual loops');
print('nested:true');

var captured = [];
function testIndex(proceed, i) { return proceed && i < 2; }
function* cells() {
  for (let i = 0; testIndex(yield ('test:' + i), i); i = yield ('update:' + i)) {
    captured.push(function () { return i; });
    yield ('body:' + i);
  }
  return captured[0]() + ',' + captured[1]();
}
var cellIterator = cells();
assert(cellIterator.next().value === 'test:0', 'first test');
assert(cellIterator.next(true).value === 'body:0', 'first body');
assert(cellIterator.next().value === 'update:0', 'first update');
gc();
assert(captured[0]() === 0, 'first captured iteration before resumed update');
assert(cellIterator.next(1).value === 'test:1', 'resumed update precedes next test');
assert(cellIterator.next(true).value === 'body:1', 'second body');
assert(cellIterator.next().value === 'update:1', 'second update');
gc();
assert(captured[0]() === 0 && captured[1]() === 1, 'captured cells remain distinct');
assert(cellIterator.next(2).value === 'test:2', 'second resumed update');
var cellsDone = cellIterator.next(false);
assert(cellsDone.done && cellsDone.value === '0,1', 'saved closures retain their own iteration');
print('cells:true');

function* abrupt() {
  try {
    for (;;) {
      try {
        yield 'a';
        yield 'b';
      } finally {
        yield 'inner';
      }
    }
  } finally {
    yield 'outer';
  }
}
var token = { value: 9007199254740993n };
token.self = token;
var returning = abrupt();
assert(returning.next().value === 'a', 'Return fixture entered body');
assert(returning.return(token).value === 'inner', 'Return retains inner finalizer');
gc();
assert(returning.next().value === 'outer', 'Return retains outer finalizer');
var returned = returning.next();
assert(returned.done && returned.value === token && returned.value.self === token,
       'whole Return identity survives both yielding finalizers');
print('return:true');
var throwing = abrupt();
assert(throwing.next().value === 'a', 'Throw fixture entered body');
assert(throwing.throw(token).value === 'inner', 'Throw retains inner finalizer');
assert(throwing.next().value === 'outer', 'Throw retains outer finalizer');
gc();
var caught = null;
try { throwing.next(); } catch (error) { caught = error; }
assert(caught === token && caught.value === 9007199254740993n,
       'whole Throw identity survives both yielding finalizers');
print('throw:true');
