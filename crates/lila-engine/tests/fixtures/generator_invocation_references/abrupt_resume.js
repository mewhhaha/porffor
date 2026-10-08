var marker = {};
var first = { [Symbol.iterator]: function () {
  print('first');
  var done = false;
  return { next: function () { var previous = done; done = true; return { done: previous, value: 1 }; } };
} };
var later = { [Symbol.iterator]: function () { print('unexpected-later'); throw marker; } };
function call() { print('unexpected-call'); }
function* values() { return call(...first, yield 'pending', ...later); }
var thrown = values();
var returned = values();
print(thrown.next().value);
try { thrown.throw(marker); } catch (error) { print(error === marker); }
print(returned.next().value);
var result = returned.return(23);
print(result.value + ':' + result.done);
function* construct() { return new call(yield 'construct?', ...later); }
var iterator = construct();
print(iterator.next().value);
try { iterator.throw(marker); } catch (error) { print(error === marker); }
function* tag() { return call`head${yield 'tag?'}tail${call()}`; }
iterator = tag();
print(iterator.next().value);
result = iterator.return(24);
print(result.value + ':' + result.done);
