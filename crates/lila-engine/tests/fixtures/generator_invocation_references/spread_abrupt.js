var marker = {};
var source = {
  [Symbol.iterator]: function () {
    print('iterator');
    return {
      next: function () { print('next'); return { done: false, get value() { print('value'); throw marker; } }; },
      return: function () { print('unexpected-close'); return {}; }
    };
  }
};
var receiver = { get method() { print('get'); return 1; } };
function later() { print('unexpected-later'); }
function* values() { return receiver.method(...source, yield 'unexpected-yield', later()); }
try { values().next(); } catch (error) { print(error === marker); }

var finite = { [Symbol.iterator]: function () {
  print('finite-iterator');
  var done = false;
  return { next: function () { var previous = done; done = true; return { done: previous, value: 2 }; } };
} };
function after() { print('after'); return 3; }
function* noncallable() { return receiver.method(yield 'pending', ...finite, after()); }
var iterator = noncallable();
print(iterator.next().value);
try { iterator.next(1); } catch (error) { print(error instanceof TypeError); }
