function Original(value) { print('construct:' + value); this.value = value; }
var selected = Original;
function* construct() { return yield new selected(yield 'inner?'); }
var iterator = construct();
print(iterator.next().value);
selected = function () { print('unexpected-constructor'); };
var result = iterator.next(7);
print(result.value.value + ':' + result.done);
result = iterator.next(9);
print(result.value + ':' + result.done);
var receiver = { tag: function (strings, value) { print('tag:' + value); return 'tag' + value; } };
function* tag() { yield receiver.tag`head${yield 'substitution?'}tail`; return 12; }
iterator = tag();
print(iterator.next().value);
receiver.tag = function () { print('unexpected-tag'); };
result = iterator.next(3);
print(result.value + ':' + result.done);
result = iterator.next();
print(result.value + ':' + result.done);
