function check(value, message) { if (!value) throw new Error(message); }
let calls = 0;
function combine(left, right) { calls++; return [left, right]; }
function* values(source, combine) {
  let value = 0;
  for (const item of source) { value = combine(yield 'left', yield 'right'); }
  return value;
}
const left = {};
const right = {};
const iterator = values([1], combine);
let result = iterator.next();
check(!result.done && result.value === 'left' && calls === 0, 'first retained operand');
gc();
result = iterator.next(left);
check(!result.done && result.value === 'right' && calls === 0, 'second operand follows first resume');
gc();
result = iterator.next(right);
check(result.done && calls === 1, 'one actual call after complete RHS');
check(result.value[0] === left && result.value[1] === right, 'whole raw arguments and assignment result');
print('generator-linear-assignment:ok');
