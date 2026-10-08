var trace = '';
var count = 0;
var source = { [Symbol.iterator]() {
  trace += 'i';
  return { next() {
    trace += 'n';
    count++;
    return count < 3 ? { done: false, get value() { trace += 'v'; return count; } } : { done: true };
  } };
} };
var scope = new Proxy({}, { has(object, key) { if (key === 'head') trace += 'h'; if (key === 'tail') trace += 'r'; return false; } });
with (scope) { var [head, ...tail] = source; }
var arrayOrder = trace === 'ihnvrnvn' && head === 1 && tail.length === 1 && tail[0] === 2;
trace = '';
var restScope = new Proxy({}, { has(object, key) { if (key === 'rest') trace += 'r'; return false; } });
var objectSource = { get p() { trace += 'g'; return 9; } };
with (restScope) { var { ...rest } = objectSource; }
arrayOrder && trace === 'rg' && rest.p === 9;
