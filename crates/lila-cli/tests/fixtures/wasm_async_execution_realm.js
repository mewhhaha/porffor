function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

var other = __lilaCreateRealm().global;

async function entryAsync(value) {
  await value;
  var nonCallableApply = new Proxy(function() {}, { apply: 0 });
  try {
    nonCallableApply();
  } catch (error) {
    assertSame(
      Object.getPrototypeOf(error),
      TypeError.prototype,
      "async post-await TypeError Realm"
    );
    return "async-entry-realm";
  }
  throw "async post-await Proxy call did not throw";
}

async function* entryAsyncGenerator(value) {
  await value;
  var nonCallableApply = new Proxy(function() {}, { apply: 0 });
  try {
    nonCallableApply();
  } catch (error) {
    assertSame(
      Object.getPrototypeOf(error),
      TypeError.prototype,
      "async-generator post-await TypeError Realm"
    );
    yield "async-generator-entry-realm";
    return;
  }
  throw "async-generator post-await Proxy call did not throw";
}

var asyncThenable = {
  then: function(resolve, reject) {
    assertSame(
      Object.getPrototypeOf(resolve),
      Function.prototype,
      "async await resolve callback Function prototype"
    );
    assertSame(
      Object.getPrototypeOf(reject),
      Function.prototype,
      "async await reject callback Function prototype"
    );
    resolve(41);
  }
};
var invokeEntryAsyncInOtherRealm = other.Array.prototype.map.bind(
  [asyncThenable],
  entryAsync
);

other.Promise.resolve(0).then(invokeEntryAsyncInOtherRealm).then(function(results) {
  var result = results[0];
  assertSame(
    Object.getPrototypeOf(result),
    Promise.prototype,
    "async invocation Promise prototype"
  );
  return result;
}).then(function(value) {
  assertSame(value, "async-entry-realm", "async captured reaction Realm");

  var generatorThenable = {
    then: function(resolve, reject) {
      assertSame(
        Object.getPrototypeOf(resolve),
        Function.prototype,
        "async-generator await resolve callback Function prototype"
      );
      assertSame(
        Object.getPrototypeOf(reject),
        Function.prototype,
        "async-generator await reject callback Function prototype"
      );
      resolve(50);
    }
  };
  var invokeEntryGeneratorInOtherRealm = other.Array.prototype.map.bind(
    [generatorThenable],
    entryAsyncGenerator
  );
  return other.Promise.resolve(0).then(invokeEntryGeneratorInOtherRealm);
}).then(function(generators) {
  var generator = generators[0];
  assertSame(
    Object.getPrototypeOf(generator),
    entryAsyncGenerator.prototype,
    "async-generator activation function Realm"
  );
  var invokeEntryGeneratorNextInOtherRealm = other.Array.prototype.map.bind(
    [undefined],
    generator.next,
    generator
  );
  return other.Promise.resolve(0).then(invokeEntryGeneratorNextInOtherRealm);
}).then(function(requests) {
  var request = requests[0];
  assertSame(
    Object.getPrototypeOf(request),
    Promise.prototype,
    "async-generator request Promise defining Realm"
  );
  return request;
}).then(function(step) {
  assertSame(
    step.value,
    "async-generator-entry-realm",
    "async-generator captured reaction Realm"
  );
  assertSame(step.done, false, "async-generator yielded completion");
  var invalidGenerator = entryAsyncGenerator(0);
  var invokeInvalidNextInOtherRealm = other.Array.prototype.map.bind(
    [undefined],
    invalidGenerator.next,
    {}
  );
  return other.Promise.resolve(0).then(invokeInvalidNextInOtherRealm);
}).then(function(requests) {
  var invalidRequest = requests[0];
  assertSame(
    Object.getPrototypeOf(invalidRequest),
    Promise.prototype,
    "invalid async-generator request Promise defining Realm"
  );
  return invalidRequest.then(
    function() {
      throw "invalid async-generator request fulfilled";
    },
    function(error) {
      assertSame(
        Object.getPrototypeOf(error),
        TypeError.prototype,
        "invalid async-generator request TypeError defining Realm"
      );
    }
  );
}).then(function() {
  print("async-execution-realm:ok");
});

true;
