function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

function assertDataDescriptor(object, key, value, enumerable, label) {
  var descriptor = Object.getOwnPropertyDescriptor(object, key);
  assertSame(descriptor.value, value, label + " value");
  assertSame(descriptor.writable, true, label + " writable");
  assertSame(descriptor.enumerable, enumerable, label + " enumerable");
  assertSame(descriptor.configurable, true, label + " configurable");
}

function assertSettlementRecord(record, expectedStatus, resultKey, expectedValue, prototype, label) {
  assertSame(Object.getPrototypeOf(record), prototype, label + " prototype");
  assertSame(Object.keys(record).join(","), "status," + resultKey, label + " key order");
  assertDataDescriptor(record, "status", expectedStatus, true, label + " status");
  assertDataDescriptor(record, resultKey, expectedValue, true, label + " result");
}

var remaining = 5;
function completeBranch() {
  remaining -= 1;
  if (remaining === 0) print("promise-callback-created-allocation-realm:ok");
}

var other = __lilaCreateRealm().global;
var otherObjectPrototype = other.Object.prototype;
var otherArrayPrototype = other.Array.prototype;
var otherAggregateErrorPrototype = other.AggregateError.prototype;

var all = other.Promise.all.call(Promise, [7, 8]);
assertSame(Object.getPrototypeOf(all), Promise.prototype, "Promise.all outer Promise realm");
all.then(function(values) {
  assertSame(Object.getPrototypeOf(values), otherArrayPrototype, "Promise.all result array prototype");
  assertSame(values.join(","), "7,8", "Promise.all result values");
  completeBranch();
}, function() {
  throw "Promise.all rejected";
});

var standard = other.Promise.allSettled.call(Promise, [
  11,
  Promise.reject("standard rejected")
]);
assertSame(Object.getPrototypeOf(standard), Promise.prototype, "standard outer Promise realm");
standard.then(function(records) {
  assertSame(Object.getPrototypeOf(records), otherArrayPrototype, "allSettled result array prototype");
  assertSettlementRecord(
    records[0],
    "fulfilled",
    "value",
    11,
    otherObjectPrototype,
    "standard fulfilled"
  );
  assertSettlementRecord(
    records[1],
    "rejected",
    "reason",
    "standard rejected",
    otherObjectPrototype,
    "standard rejected"
  );
  completeBranch();
}, function() {
  throw "standard allSettled rejected";
});

var keyed = other.Promise.allSettledKeyed.call(Promise, {
  fulfilled: 22,
  rejected: Promise.reject("keyed rejected")
});
assertSame(Object.getPrototypeOf(keyed), Promise.prototype, "keyed outer Promise realm");
keyed.then(function(records) {
  assertSame(Object.getPrototypeOf(records), null, "keyed outer result prototype");
  assertSettlementRecord(
    records.fulfilled,
    "fulfilled",
    "value",
    22,
    otherObjectPrototype,
    "keyed fulfilled"
  );
  assertSettlementRecord(
    records.rejected,
    "rejected",
    "reason",
    "keyed rejected",
    otherObjectPrototype,
    "keyed rejected"
  );
  completeBranch();
}, function() {
  throw "keyed allSettled rejected";
});

var nonemptyAny = other.Promise.any.call(Promise, [
  Promise.reject("first error"),
  Promise.reject("second error")
]);
assertSame(Object.getPrototypeOf(nonemptyAny), Promise.prototype, "nonempty any outer Promise realm");
nonemptyAny.then(function() {
  throw "nonempty any fulfilled";
}, function(error) {
  assertSame(Object.getPrototypeOf(error), otherAggregateErrorPrototype, "nonempty any error prototype");
  assertSame(Object.getPrototypeOf(error.errors), otherArrayPrototype, "nonempty any errors array prototype");
  assertSame(error.errors.length, 2, "nonempty any errors length");
  assertSame(error.errors[0], "first error", "nonempty any first error");
  assertSame(error.errors[1], "second error", "nonempty any second error");
  assertDataDescriptor(error, "errors", error.errors, false, "nonempty any errors");
  completeBranch();
});

var emptyAny = other.Promise.any.call(Promise, []);
assertSame(Object.getPrototypeOf(emptyAny), Promise.prototype, "empty any outer Promise realm");
emptyAny.then(function() {
  throw "empty any fulfilled";
}, function(error) {
  assertSame(Object.getPrototypeOf(error), otherAggregateErrorPrototype, "empty any error prototype");
  assertSame(Object.getPrototypeOf(error.errors), otherArrayPrototype, "empty any errors array prototype");
  assertSame(error.errors.length, 0, "empty any errors length");
  assertDataDescriptor(error, "errors", error.errors, false, "empty any errors");
  completeBranch();
});

true;
