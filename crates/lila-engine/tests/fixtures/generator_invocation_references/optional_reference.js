var original = {
  value: 10,
  get method() { print('get'); return function (value) { print('call:' + (this === original) + ':' + this.value + ':' + value); return value; }; },
  get tag() { print('tag-get'); return function (strings, value) { print('tag:' + (this === original) + ':' + value); return value; }; },
  factory: function () { print('factory'); return function (value) { 'use strict'; print('plain:' + (this === undefined) + ':' + value); return value; }; }
};
var selected = original;
function* values() { return (selected?.method)(yield 'optional?'); }
var iterator = values();
print(iterator.next().value);
selected = { method: function () { print('unexpected-method'); } };
original.value = 20;
var result = iterator.next(7);
print(result.value + ':' + result.done);
function key() { print('unexpected-key'); return 'method'; }
function after() { print('after'); return 2; }
function* nullish() { return (null?.[key()])(yield 'nullish?', after()); }
iterator = nullish();
print(iterator.next().value);
try { iterator.next(1); } catch (error) { print(error instanceof TypeError); }
function* plain() { return (original?.factory())(yield 'plain?'); }
iterator = plain();
print(iterator.next().value);
result = iterator.next(8);
print(result.value + ':' + result.done);
function* tag() { return (original?.tag)`head${yield 'tag?'}tail`; }
iterator = tag();
print(iterator.next().value);
result = iterator.next(9);
print(result.value + ':' + result.done);
