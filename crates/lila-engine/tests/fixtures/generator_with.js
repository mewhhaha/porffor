function check(condition, label) { if (!condition) throw label; }
function next(iterator, sent, expected, done, label) {
  var result = iterator.next(sent);
  check(result.value === expected && result.done === done, label);
  return result;
}
var value = 100, headRuns = 0;
function* headAndBody() {
  try {
    with ((headRuns++, yield 'head-one', yield 'head-two')) { yield value; }
  } finally { yield value; }
}
var iterator = headAndBody(), scope = {value: 5};
next(iterator, undefined, 'head-one', false, 'outer-head-first');
gc();
next(iterator, scope, 'head-two', false, 'outer-head-second');
gc();
next(iterator, scope, 5, false, 'completed-head-enters-once');
next(iterator, undefined, 100, false, 'normal-with-exit-before-finally');
next(iterator, undefined, undefined, true, 'normal-head-done');
check(headRuns === 1, 'head-no-replay');
var whole = {returnValue: 17};
iterator = headAndBody();
next(iterator, undefined, 'head-one', false, 'unfinished-head');
var result = iterator.return(whole);
check(result.value === 100 && !result.done, 'head-return-never-enters-with');
next(iterator, undefined, whole, true, 'head-return-whole');

function* nestedScopes(outer, inner) {
  with (outer) {
    let local = 7;
    yield function readOuter() { return [value, local]; };
    with (inner) {
      yield function readInner() { return [value, local]; };
      value = yield 'write';
    }
    yield value;
  }
  return value;
}
var outer = {value: 1}, inner = {value: 2};
iterator = nestedScopes(outer, inner);
var readOuter = iterator.next().value;
check(readOuter()[0] === 1 && readOuter()[1] === 7, 'outer-closure-original-cells');
gc();
var readInner = iterator.next().value;
check(readInner()[0] === 2 && readInner()[1] === 7, 'inner-closure-parent-hops');
outer.value = 3; inner.value = 4;
next(iterator, undefined, 'write', false, 'inner-original-reference');
delete inner.value;
gc();
next(iterator, 9, 3, false, 'inner-put-and-parent-reattach');
check(inner.value === 9 && value === 100, 'original-selected-object-after-delete');
next(iterator, undefined, 100, true, 'nested-normal-restores-outer');
gc();
check(readOuter()[0] === 3 && readInner()[0] === 9 && readInner()[1] === 7,
      'escaping-closures-retain-after-with-exit');

function* eagerClosure(object) {
  var read;
  with (object) { let local = 11; read = function () { return [value, local]; }; }
  return read;
}
iterator = eagerClosure(outer); result = iterator.next();
check(result.done, 'eager-with-has-owned-cleanup');
var readEager = result.value;
gc(); outer.value = 13;
check(readEager()[0] === 13 && readEager()[1] === 11 && value === 100,
      'eager-with-real-record-capture');

function* eagerEnclosure(choice, object) {
  var read;
  if (choice) { with (object) { read = () => value; } }
  else { switch (choice) { case 0: with (object) { read = () => value; } break; default: read = () => value; } }
  return read;
}
for (var choice of [true, 0]) {
  result = eagerEnclosure(choice, outer).next();
  check(result.done && result.value() === 13, 'eager-enclosing-if-switch-phase-ownership');
  gc();
}
function* oldIteratorBody(view) { for (let item of [1]) { with (view) { item; } yield item; } }
iterator = oldIteratorBody({item: 99});
next(iterator, undefined, 1, false, 'iterator-eager-with-retains-old-route');
next(iterator, undefined, undefined, true, 'iterator-eager-with-old-completion');

function* primitiveHead() { with ('abc') { yield length; yield length; } return value; }
iterator = primitiveHead();
next(iterator, undefined, 3, false, 'to-object-primitive');
gc(); next(iterator, undefined, 3, false, 'same-primitive-box-resume');
next(iterator, undefined, 100, true, 'primitive-parent-restored');
function* nullHead() { with (yield 'null-head') { throw 'body-ran'; } }
iterator = nullHead(); next(iterator, undefined, 'null-head', false, 'null-head-prefix');
var nullError;
try { iterator.next(null); } catch (error) { nullError = error; }
check(nullError instanceof TypeError, 'to-object-failure-before-environment');

function* lexicalTdz(object) {
  try { with (object) { yield value; let value = yield 'unreached'; } }
  catch (error) { yield error instanceof ReferenceError; }
  return value;
}
iterator = lexicalTdz(scope);
next(iterator, undefined, true, false, 'source-block-tdz-before-with-lookup');
next(iterator, undefined, 100, true, 'caught-tdz-restores-parent');

function* abruptBody(object) {
  try { with (object) { yield value; } }
  finally { yield value; }
}
iterator = abruptBody(scope); next(iterator, undefined, 5, false, 'return-inside-with');
gc(); result = iterator.return(whole);
check(result.value === 100 && !result.done, 'return-restores-before-finally');
next(iterator, undefined, whole, true, 'return-identity-after-finally');
var thrown = {throwValue: 23};
iterator = abruptBody(scope); next(iterator, undefined, 5, false, 'throw-inside-with');
result = iterator.throw(thrown);
check(result.value === 100 && !result.done, 'throw-restores-before-finally');
var observed;
try { iterator.next(); } catch (error) { observed = error; }
check(observed === thrown, 'throw-whole-after-finally');

var finalized = [];
function* control(object) {
  outerLoop: for (let i = 0; i < 2; i++) {
    with (object) {
      switch (yield 'discriminant') {
        case yield 'selector':
          try { yield value; continue outerLoop; }
          finally { finalized.push(value); yield 'inside-finally'; }
        default: throw 'wrong-case';
      }
    }
  }
  stop: with (object) { yield value; break stop; }
  return value;
}
iterator = control(scope);
for (var round = 0; round < 2; round++) {
  next(iterator, undefined, 'discriminant', false, 'switch-discriminant');
  next(iterator, 0, 'selector', false, 'switch-selector');
  next(iterator, 0, 5, false, 'switch-case-under-original-with');
  next(iterator, undefined, 'inside-finally', false, 'continue-inner-finally-first');
  gc();
}
next(iterator, undefined, 5, false, 'labelled-with-body');
next(iterator, undefined, 100, true, 'label-break-restores-parent');
check(finalized.join(',') === '5,5', 'each-continue-original-environment');

function* delegate() { yield 'delegated'; return 31; }
function* delegatedBody(object) { with (object) { var received = yield* delegate(); yield [value,received]; } return value; }
iterator = delegatedBody(scope); next(iterator, undefined, 'delegated', false, 'delegate-retains-with');
gc(); result = iterator.next();
check(result.value[0] === 5 && result.value[1] === 31 && !result.done, 'delegate-completion-original-record');
next(iterator, undefined, 100, true, 'delegate-exit-parent');

var lookups = 0, blocked = false;
var dynamicScope = {value: 41};
Object.defineProperty(dynamicScope, Symbol.unscopables, {get: function () {
  lookups++; return {value: blocked};
}});
function* dynamicLookup(object) { with (object) { yield value; yield value; } return value; }
iterator = dynamicLookup(dynamicScope);
next(iterator, undefined, 41, false, 'unscopables-first');
check(lookups === 1, 'first-hasbinding-once');
blocked = true; gc();
next(iterator, undefined, 100, false, 'unscopables-rechecked-after-resume');
check(lookups === 2, 'new-reference-hasbinding-once');
next(iterator, undefined, 100, true, 'dynamic-environment-retired');
print('generator-with:ok');
