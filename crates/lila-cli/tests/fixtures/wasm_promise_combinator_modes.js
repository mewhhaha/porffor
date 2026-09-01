function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

function assertSettlement(record, status, resultKey, expected, label) {
  assertSame(record.status, status, label + " status");
  assertSame(record[resultKey], expected, label + " result");
}

var remaining = 8;
function complete() {
  remaining -= 1;
  if (remaining === 0) print("promise-combinator-modes:ok");
}

Promise.all([Promise.resolve(1), 2]).then(function (values) {
  assertSame(values.join(","), "1,2", "all values");
  complete();
}, function () {
  throw "all unexpectedly rejected";
});

Promise.all([Promise.reject("all rejection")]).then(function () {
  throw "all unexpectedly fulfilled";
}, function (reason) {
  assertSame(reason, "all rejection", "all rejection");
  complete();
});

Promise.allSettled([3, Promise.reject("settled rejection")]).then(function (records) {
  assertSettlement(records[0], "fulfilled", "value", 3, "allSettled fulfilled");
  assertSettlement(records[1], "rejected", "reason", "settled rejection", "allSettled rejected");
  complete();
}, function () {
  throw "allSettled unexpectedly rejected";
});

Promise.any([Promise.reject("ignored"), Promise.resolve(4)]).then(function (value) {
  assertSame(value, 4, "any first fulfillment");
  complete();
}, function () {
  throw "any unexpectedly rejected";
});

Promise.any([Promise.reject("first"), Promise.reject("second")]).then(function () {
  throw "any unexpectedly fulfilled";
}, function (error) {
  assertSame(error.errors.join(","), "first,second", "any rejection order");
  complete();
});

Promise.allKeyed({ first: Promise.resolve(5), second: 6 }).then(function (values) {
  assertSame(Object.getPrototypeOf(values), null, "allKeyed prototype");
  assertSame(values.first, 5, "allKeyed first");
  assertSame(values.second, 6, "allKeyed second");
  complete();
}, function () {
  throw "allKeyed unexpectedly rejected";
});

Promise.allKeyed({ rejected: Promise.reject("keyed rejection") }).then(function () {
  throw "allKeyed rejection unexpectedly fulfilled";
}, function (reason) {
  assertSame(reason, "keyed rejection", "allKeyed rejection");
  complete();
});

Promise.allSettledKeyed({ fulfilled: 7, rejected: Promise.reject("keyed settled") }).then(function (records) {
  assertSame(Object.getPrototypeOf(records), null, "allSettledKeyed prototype");
  assertSettlement(records.fulfilled, "fulfilled", "value", 7, "allSettledKeyed fulfilled");
  assertSettlement(records.rejected, "rejected", "reason", "keyed settled", "allSettledKeyed rejected");
  complete();
}, function () {
  throw "allSettledKeyed unexpectedly rejected";
});

true;
