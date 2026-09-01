function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

var other = __lilaCreateRealm().global;
var remaining = 6;

function observe(methodName, promise) {
  assertSame(Object.getPrototypeOf(promise), Promise.prototype, methodName + " Promise Realm");
  promise.then(function () {
    throw methodName + " unexpectedly fulfilled";
  }, function (error) {
    assertSame(
      Object.getPrototypeOf(error),
      other.TypeError.prototype,
      methodName + " TypeError Realm"
    );
    remaining -= 1;
    if (remaining === 0) print("promise-combinator-algorithm-error-realm:ok");
  });
}

observe("Promise.all", other.Promise.all.call(Promise, null));
observe("Promise.allSettled", other.Promise.allSettled.call(Promise, null));
observe("Promise.allKeyed", other.Promise.allKeyed.call(Promise, 0));
observe("Promise.allSettledKeyed", other.Promise.allSettledKeyed.call(Promise, 0));
observe("Promise.any", other.Promise.any.call(Promise, null));
observe("Promise.race", other.Promise.race.call(Promise, null));

true;
