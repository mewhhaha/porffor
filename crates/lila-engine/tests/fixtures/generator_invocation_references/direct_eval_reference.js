function* values() { var local = 41; return eval('local + 1', yield 'eval?'); }
var iterator = values();
print(iterator.next().value);
globalThis.eval = function () { print('unexpected-global-eval'); return 0; };
var result = iterator.next(0);
print(result.value + ':' + result.done);
function* custom(eval) { return eval(yield 'custom?'); }
iterator = custom(function (value) { 'use strict'; print('custom:' + (this === undefined) + ':' + value); return value; });
print(iterator.next().value);
result = iterator.next(7);
print(result.value + ':' + result.done);
