let $262 = { createRealm: __porfCreateRealm };
let other = $262.createRealm().global;
let values = [undefined, null, true, "x", Symbol("x"), 7];

for (const value of values) {
  let C = new other.Function();
  C.prototype = value;
  let result = Reflect.construct(AggregateError, [[]], C);
  if (Object.getPrototypeOf(result) !== other.AggregateError.prototype) {
    throw "cross-realm AggregateError prototype fallback";
  }
}

if (other.AggregateError.prototype === AggregateError.prototype) {
  throw "realm AggregateError prototype identity";
}

123;
