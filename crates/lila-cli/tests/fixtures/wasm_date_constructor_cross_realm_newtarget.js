let $262 = { createRealm: __lilaCreateRealm };
let other = $262.createRealm().global;

function assertOtherRealmDate(args, expectedTime, label) {
  let C = new other.Function();
  C.prototype = null;
  let value = Reflect.construct(Date, args, C);

  if (Object.getPrototypeOf(value) !== other.Date.prototype) {
    throw label + " realm prototype";
  }
  let time = Date.prototype.getTime.call(value);
  if (expectedTime === undefined) {
    if (time !== time) throw label + " Date brand/value";
  } else if (time !== expectedTime) {
    throw label + " Date brand/value";
  }
}

assertOtherRealmDate([], undefined, "zero");
assertOtherRealmDate([0], 0, "one");
assertOtherRealmDate([1970, 0], undefined, "multiple");

function assertCustomPrototype(prototype, time, label) {
  let C = new other.Function();
  C.prototype = prototype;
  let value = Reflect.construct(Date, [time], C);
  if (Object.getPrototypeOf(value) !== prototype) {
    throw label + " custom prototype tag/identity";
  }
  if (Date.prototype.getTime.call(value) !== time) {
    throw label + " custom prototype Date brand/value";
  }
}

assertCustomPrototype({ custom: true }, 1, "Object");
assertCustomPrototype(function () {}, 2, "Function");
assertCustomPrototype([], 3, "Array");

let oneOrder = [];
let oneNewTarget = new Proxy(new other.Function(), {
  get: function (target, key, receiver) {
    if (key === "prototype") {
      oneOrder.push("prototype");
      return null;
    }
    return Reflect.get(target, key, receiver);
  },
});
let oneArgument = {
  valueOf: function () {
    oneOrder.push("value");
    return 4;
  },
};
let orderedOne = Reflect.construct(Date, [oneArgument], oneNewTarget);
if (!(oneOrder.join(",") === "value,prototype" &&
      Object.getPrototypeOf(orderedOne) === other.Date.prototype &&
      Date.prototype.getTime.call(orderedOne) === 4)) {
  throw "one-argument value/prototype order";
}

let multipleOrder = [];
let multipleNewTarget = new Proxy(new other.Function(), {
  get: function (target, key, receiver) {
    if (key === "prototype") {
      multipleOrder.push("prototype");
      return null;
    }
    return Reflect.get(target, key, receiver);
  },
});
let orderedYear = {
  valueOf: function () {
    multipleOrder.push("year");
    return 1970;
  },
};
let orderedMonth = {
  valueOf: function () {
    multipleOrder.push("month");
    return 0;
  },
};
let orderedMultiple = Reflect.construct(
  Date,
  [orderedYear, orderedMonth],
  multipleNewTarget,
);
let orderedMultipleTime = Date.prototype.getTime.call(orderedMultiple);
if (!(multipleOrder.join(",") === "year,month,prototype" &&
      Object.getPrototypeOf(orderedMultiple) === other.Date.prototype &&
      orderedMultipleTime === orderedMultipleTime)) {
  throw "multiple-argument coercion/prototype order";
}

let revocable;
revocable = Proxy.revocable(new other.Function(), {
  get: function (target, key, receiver) {
    if (key === "prototype") {
      revocable.revoke();
      return null;
    }
    return Reflect.get(target, key, receiver);
  },
});
let revocationThrew = false;
try {
  Reflect.construct(Date, [3], revocable.proxy);
} catch (error) {
  revocationThrew = error instanceof TypeError;
}
if (!revocationThrew) throw "revoked function realm fallback";

262;
