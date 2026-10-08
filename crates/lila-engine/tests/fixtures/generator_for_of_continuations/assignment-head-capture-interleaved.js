function same(actual, expected, label) {
  if (actual !== expected) throw label + ': ' + actual + ' !== ' + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + ' value');
  same(result.done, done, label + ' done');
}
function source(values, state) {
  return { [Symbol.iterator]() {
    return {
      next() {
        const index = state.next++;
        return index < values.length ? { value: values[index], done: false } : { done: true };
      },
      return() { ++state.close; return {}; }
    };
  } };
}
var target = -1;
function* walk(iterable) {
  for (target of iterable) {
    let body = target * 10;
    const readers = [() => target, () => body];
    yield readers;
    ++body;
    yield readers;
  }
  return target;
}
const firstState = { next: 0, close: 0 };
const secondState = { next: 0, close: 0 };
const first = walk(source([1, 2], firstState));
const second = walk(source([3, 4], secondState));
const firstInitial = first.next().value;
same(firstInitial[0](), 1, 'first head writes outer target');
same(firstInitial[1](), 10, 'first body capture');
const secondInitial = second.next().value;
same(secondInitial[0](), 3, 'second head writes same target');
same(firstInitial[0](), 3, 'first capture sees other owner assignment');
same(firstInitial[1](), 10, 'first body is not shared');
target = 100;
step(first.next(), firstInitial, false, 'first second yield');
same(firstInitial[0](), 100, 'first resume does not replay head assignment');
same(firstInitial[1](), 11, 'first body continuation once');
same(firstState.next, 1, 'first does not step across body yield');
step(second.next(), secondInitial, false, 'second second yield');
same(secondInitial[0](), 100, 'second resume retains outer mutation');
same(secondInitial[1](), 31, 'second body continuation once');
same(secondState.next, 1, 'second does not step across body yield');
target = 200;
const firstLater = first.next().value;
same(target, 2, 'next first iteration assigns once');
same(firstLater === firstInitial, false, 'fresh body readers');
same(firstLater[1](), 20, 'fresh first body cell');
same(firstInitial[0](), 2, 'old capture still names outer target');
same(firstInitial[1](), 11, 'old first body retained');
const secondLater = second.next().value;
same(target, 4, 'next second iteration assigns once');
same(secondLater[1](), 40, 'fresh second body cell');
same(firstLater[0](), 4, 'all captures share target binding');
same(secondInitial[1](), 31, 'old second body retained');
target = 500;
step(first.next(), firstLater, false, 'later first second yield');
step(second.next(), secondLater, false, 'later second second yield');
same(firstLater[0](), 500, 'later first target mutation retained');
same(secondLater[0](), 500, 'later second target mutation retained');
same(firstLater[1](), 21, 'later first body continuation');
same(secondLater[1](), 41, 'later second body continuation');
step(first.next(), 500, true, 'first exhausts');
step(second.next(), 500, true, 'second exhausts');
same(firstState.next, 3, 'first exact iteration steps');
same(secondState.next, 3, 'second exact iteration steps');
same(firstState.close, 0, 'normal first exhaustion does not close');
same(secondState.close, 0, 'normal second exhaustion does not close');
same(firstInitial[0](), 500, 'outer capture after completion');
same(secondInitial[1](), 31, 'earlier body capture after completion');
step(first.next(), undefined, true, 'first remains completed');
step(second.next(), undefined, true, 'second remains completed');
same(firstState.next, 3, 'completed first cannot assign or step');
same(secondState.next, 3, 'completed second cannot assign or step');
print('ok');
262;
