function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

var other = __lilaCreateRealm().global;

function expectOtherTypeError(action, expectedMessage, label) {
  var threw = false;
  try {
    action();
  } catch (error) {
    threw = true;
    assert(error instanceof other.TypeError, label + " defining realm");
    assert(!(error instanceof TypeError), label + " not entry realm");
    assert(error.message === expectedMessage, label + " message");
  }
  assert(threw, label + " did not throw");
}

expectOtherTypeError(
  function () { other.Map(); },
  "Map constructor requires new",
  "Map requires new"
);
expectOtherTypeError(
  function () { other.Set(); },
  "Set constructor requires new",
  "Set requires new"
);

var savedMapSet = other.Map.prototype.set;
other.Map.prototype.set = 0;
expectOtherTypeError(
  function () { new other.Map([[1, 2]]); },
  "Map constructor set method is not callable",
  "Map setter"
);
other.Map.prototype.set = savedMapSet;

var savedSetAdd = other.Set.prototype.add;
other.Set.prototype.add = 0;
expectOtherTypeError(
  function () { new other.Set([1]); },
  "Set constructor add method is not callable",
  "Set adder"
);
other.Set.prototype.add = savedSetAdd;

function withIteratorMethod(method) {
  var iterable = {};
  iterable[Symbol.iterator] = method;
  return iterable;
}

var nonCallableIterator = withIteratorMethod(0);
expectOtherTypeError(
  function () { new other.Map(nonCallableIterator); },
  "Map constructor iterator method is not callable",
  "Map iterator method"
);
expectOtherTypeError(
  function () { new other.Set(nonCallableIterator); },
  "Set constructor iterator method is not callable",
  "Set iterator method"
);

var primitiveIterator = withIteratorMethod(function () { return 0; });
expectOtherTypeError(
  function () { new other.Map(primitiveIterator); },
  "Map constructor iterator method must return an object",
  "Map iterator result"
);
expectOtherTypeError(
  function () { new other.Set(primitiveIterator); },
  "Set constructor iterator method must return an object",
  "Set iterator result"
);

var nonCallableNext = withIteratorMethod(function () { return { next: 0 }; });
expectOtherTypeError(
  function () { new other.Map(nonCallableNext); },
  "Map constructor iterator next method is not callable",
  "Map next"
);
expectOtherTypeError(
  function () { new other.Set(nonCallableNext); },
  "Set constructor iterator next method is not callable",
  "Set next"
);

var primitiveNextResult = withIteratorMethod(function () {
  return { next: function () { return 0; } };
});
expectOtherTypeError(
  function () { new other.Map(primitiveNextResult); },
  "Map constructor iterator next result must be an object",
  "Map next result"
);
expectOtherTypeError(
  function () { new other.Set(primitiveNextResult); },
  "Set constructor iterator next result must be an object",
  "Set next result"
);

var primitiveMapEntry = withIteratorMethod(function () {
  var first = true;
  return {
    next: function () {
      if (first) {
        first = false;
        return { value: 0, done: false };
      }
      return { value: undefined, done: true };
    }
  };
});
expectOtherTypeError(
  function () { new other.Map(primitiveMapEntry); },
  "Map constructor iterator value must be an object",
  "Map entry"
);

expectOtherTypeError(
  function () { other.Map.prototype.forEach.call(new Map(), 0); },
  "Map.prototype.forEach callback must be callable",
  "Map forEach"
);
expectOtherTypeError(
  function () { other.Set.prototype.forEach.call(new Set(), 0); },
  "Set.prototype.forEach callback must be callable",
  "Set forEach"
);

var marker = {};
var abruptIterable = {};
Object.defineProperty(abruptIterable, Symbol.iterator, {
  get: function () { throw marker; }
});
var abruptThrew = false;
try {
  new other.Map(abruptIterable);
} catch (error) {
  abruptThrew = true;
  assert(error === marker, "iterator getter abrupt identity");
}
assert(abruptThrew, "iterator getter did not throw");

assert(new other.Map([[1, 2]]).get(1) === 2, "Map success control");
assert(new other.Set([3]).has(3), "Set success control");

true;
