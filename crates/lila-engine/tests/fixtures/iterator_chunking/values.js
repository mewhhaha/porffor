var token = {}, symbol = Symbol(), values = [token, 7n, symbol, undefined, null];
var n = 0, gets = 0;
var source = {get next() { ++gets; return function () { return n < values.length ? {value: values[n++], done: false} : {done: true}; }; }};
var chunks = Iterator.prototype.chunks.call(source, 2);
Object.defineProperty(source, 'next', {value() { throw 9; }});
check(gets === 1 && n === 0);
var first = chunks.next().value, second = chunks.next().value, last = chunks.next().value;
check(first.length === 2 && first[0] === token && first[1] === 7n);
check(second.length === 2 && second[0] === symbol && second[1] === undefined);
check(last.length === 1 && last[0] === null && first !== second && last !== first && chunks.next().done);
for (var mode of ['only-full', 'allow-partial']) {
  var windows = [1, 2, 3, 4].values().windows(3, mode);
  var one = windows.next().value;
  one[1] = 99;
  var two = windows.next().value;
  check(one !== two && two.join(',') === '2,3,4' && windows.next().done && windows.next().done);
}
check([1, 2].values().windows(3).next().done);
var partial = [1, 2].values().windows(3, 'allow-partial');
check(partial.next().value.join(',') === '1,2' && partial.next().done);
check([].values().windows(3, 'allow-partial').next().done);
check([1, 2].values().windows(1).toArray().map(function (array) { return array[0]; }).join(',') === '1,2');
check([1, 2, 3].values().chunks(1).toArray().length === 3);
var helper = Iterator.prototype.chunks.call({next() { return {value: 8, done: false}; }}, 1);
var writes = 0;
Object.defineProperty(Array.prototype, '0', {set() { ++writes; }, configurable: true});
var result = helper.next();
delete Array.prototype[0];
check(result.value[0] === 8 && Object.hasOwn(result.value, '0') && writes === 0);
