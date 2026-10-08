function report(result) { print(result.value + ':' + result.done); }
function chooseTest(a, b) { return a && b; }
function nextIndex(i, a, b) { return i + a + b; }
function* phases() {
  var i;
  for (i = yield 'init-a'; chooseTest(yield ('test-a:' + i), yield ('test-b:' + i));
       i = nextIndex(i, yield ('update-a:' + i), yield ('update-b:' + i))) {
    if (i === 0) {
      yield ('body-a:' + i);
      yield ('body-b:' + i);
    } else {
      yield ('else:' + i);
      break;
    }
  }
  return 'done:' + i;
}
var phased = phases();
report(phased.next());
report(phased.next(0));
report(phased.next(true));
report(phased.next(true));
report(phased.next());
report(phased.next());
report(phased.next(1));
report(phased.next(1));
report(phased.next(true));
report(phased.next(true));
report(phased.next());

function* whilePhases() {
  var i = 0;
  while (yield ('while-test:' + i)) {
    yield ('while-a:' + i);
    yield ('while-b:' + i);
    i++;
  }
  return i;
}
var whileIterator = whilePhases();
report(whileIterator.next());
report(whileIterator.next(true));
report(whileIterator.next());
report(whileIterator.next());
report(whileIterator.next(false));

function* doPhases() {
  var i = 0;
  do {
    yield ('do-body:' + i);
    i++;
  } while (yield ('do-test:' + i));
  return i;
}
var doIterator = doPhases();
report(doIterator.next());
report(doIterator.next());
report(doIterator.next(true));
report(doIterator.next());
report(doIterator.next(false));
