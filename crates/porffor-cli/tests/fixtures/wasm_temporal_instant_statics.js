// Temporal.Instant.compare, .fromEpochMilliseconds, .fromEpochNanoseconds and
// Temporal.Instant.prototype.{toJSON,valueOf}.
//
// Every rendered value is printed rather than compared against a recomputed
// expectation, so the CLI test holds the literal ISO strings and a wrong
// rendering cannot match itself.

function expectError(errorConstructor, callback) {
  var thrown = false;
  try {
    callback();
  } catch (error) {
    if (!(error instanceof errorConstructor)) throw error;
    thrown = true;
  }
  if (!thrown) throw errorConstructor.name;
}

function comparison(value) {
  if (value === -1) return "lt";
  if (value === 0) return "eq";
  if (value === 1) return "gt";
  throw "comparison";
}

var compare = Temporal.Instant.compare;
var fromEpochMilliseconds = Temporal.Instant.fromEpochMilliseconds;
var fromEpochNanoseconds = Temporal.Instant.fromEpochNanoseconds;
var toJSON = Temporal.Instant.prototype.toJSON;
var valueOf = Temporal.Instant.prototype.valueOf;

if (typeof compare !== "function") throw "compare missing";
if (typeof fromEpochMilliseconds !== "function") throw "fromEpochMilliseconds missing";
if (typeof fromEpochNanoseconds !== "function") throw "fromEpochNanoseconds missing";
if (typeof toJSON !== "function") throw "toJSON missing";
if (typeof valueOf !== "function") throw "valueOf missing";

if (compare.name !== "compare") throw "compare name";
if (compare.length !== 2) throw "compare length";
if (fromEpochMilliseconds.name !== "fromEpochMilliseconds") throw "fromEpochMilliseconds name";
if (fromEpochMilliseconds.length !== 1) throw "fromEpochMilliseconds length";
if (fromEpochNanoseconds.name !== "fromEpochNanoseconds") throw "fromEpochNanoseconds name";
if (fromEpochNanoseconds.length !== 1) throw "fromEpochNanoseconds length";
if (toJSON.name !== "toJSON") throw "toJSON name";
if (toJSON.length !== 0) throw "toJSON length";
if (valueOf.name !== "valueOf") throw "valueOf name";
if (valueOf.length !== 0) throw "valueOf length";

// toJSON shares toString's emitter but must not share its function object.
if (toJSON === Temporal.Instant.prototype.toString) throw "toJSON identity";

var epoch = new Temporal.Instant(0n);
var early = new Temporal.Instant(-1000n);
var late = new Temporal.Instant(1000n);

// epochNanoseconds round-trips exactly, including the two-limb magnitudes the
// millisecond widening produces.
if (fromEpochMilliseconds(217175010123).epochNanoseconds !== 217175010123000000n) throw "ms post";
if (fromEpochMilliseconds(-217175010876).epochNanoseconds !== -217175010876000000n) throw "ms pre";
if (fromEpochMilliseconds(10).epochNanoseconds !== 10000000n) throw "ms small";
if (fromEpochMilliseconds(-0).epochNanoseconds !== 0n) throw "ms negative zero";
if (fromEpochNanoseconds(217175010123456789n).epochNanoseconds !== 217175010123456789n) throw "ns post";
if (fromEpochNanoseconds(-217175010876543211n).epochNanoseconds !== -217175010876543211n) throw "ns pre";
if (fromEpochNanoseconds(10n).epochNanoseconds !== 10n) throw "ns small";

// fromEpoch* are plain functions, so the receiver is never consulted.
if (Object.getPrototypeOf(fromEpochMilliseconds(10)) !== Temporal.Instant.prototype) throw "ms prototype";
if (Object.getPrototypeOf(fromEpochNanoseconds(10n)) !== Temporal.Instant.prototype) throw "ns prototype";

expectError(TypeError, function () {
  fromEpochMilliseconds(42n);
});
expectError(RangeError, function () {
  fromEpochMilliseconds();
});
expectError(RangeError, function () {
  fromEpochMilliseconds(undefined);
});
expectError(RangeError, function () {
  fromEpochMilliseconds(Infinity);
});
expectError(RangeError, function () {
  fromEpochMilliseconds(-Infinity);
});
expectError(RangeError, function () {
  fromEpochMilliseconds(NaN);
});
expectError(RangeError, function () {
  fromEpochMilliseconds(1.3);
});
expectError(RangeError, function () {
  fromEpochMilliseconds(-0.5);
});
expectError(RangeError, function () {
  fromEpochMilliseconds(8640000000000001);
});
expectError(RangeError, function () {
  fromEpochMilliseconds(-8640000000000001);
});
expectError(TypeError, function () {
  fromEpochNanoseconds();
});
expectError(TypeError, function () {
  fromEpochNanoseconds(null);
});
expectError(TypeError, function () {
  fromEpochNanoseconds(42);
});
expectError(RangeError, function () {
  fromEpochNanoseconds(8640000000000000000001n);
});
expectError(RangeError, function () {
  fromEpochNanoseconds(-8640000000000000000001n);
});
expectError(TypeError, function () {
  new fromEpochMilliseconds(0);
});
expectError(TypeError, function () {
  new fromEpochNanoseconds(0n);
});

// compare coerces both arguments through ToTemporalInstant, first one first.
expectError(RangeError, function () {
  compare({}, epoch);
});
expectError(RangeError, function () {
  compare(epoch, {});
});
expectError(TypeError, function () {
  compare(1, epoch);
});
expectError(TypeError, function () {
  compare(epoch, 1);
});
expectError(RangeError, function () {
  compare("1970-01-01T00:00", epoch);
});
expectError(TypeError, function () {
  new compare(epoch, epoch);
});

var secondArgumentWasRead = false;
expectError(RangeError, function () {
  compare("+275760-09-13T00:00:00.000-12", {
    toString: function () {
      secondArgumentWasRead = true;
      return "1970-01-01T00:00Z";
    },
  });
});
if (secondArgumentWasRead) throw "compare argument order";

// valueOf always throws, which is what makes relational operators throw too.
expectError(TypeError, function () {
  epoch.valueOf();
});
expectError(TypeError, function () {
  return epoch < epoch;
});
expectError(TypeError, function () {
  return epoch <= epoch;
});
expectError(TypeError, function () {
  return epoch > epoch;
});
expectError(TypeError, function () {
  return epoch >= epoch;
});
expectError(TypeError, function () {
  valueOf.call({});
});
expectError(TypeError, function () {
  valueOf.call(Temporal.Instant.prototype);
});
expectError(TypeError, function () {
  toJSON.call({});
});
expectError(TypeError, function () {
  toJSON.call(Temporal.Instant.prototype);
});
if ((epoch === epoch) !== true) throw "identity";
if ((epoch === late) !== false) throw "identity other";

var parts = [];
parts.push(comparison(compare(early, late)));
parts.push(comparison(compare(late, early)));
parts.push(comparison(compare(late, late)));
parts.push(comparison(compare(new Temporal.Instant(1000000000000000000n), new Temporal.Instant(500000000000000000n))));
parts.push(comparison(compare("1970-01-01T00:00Z", epoch)));
parts.push(comparison(compare(epoch, "1970-01-01T00:00+01:00")));
parts.push(comparison(compare("2016-12-31T23:59:60Z", new Temporal.Instant(1483228799000000000n))));
parts.push(fromEpochMilliseconds(0).toJSON());
parts.push(fromEpochMilliseconds(217175010123).toJSON());
parts.push(fromEpochMilliseconds(-217175010876).toJSON());
parts.push(fromEpochNanoseconds(217175010123456789n).toJSON());
parts.push(new Temporal.Instant(-13849764999999999n).toJSON());
parts.push(new Temporal.Instant(30123456000n).toJSON());
parts.push(fromEpochMilliseconds(8640000000000000).toJSON());
parts.push(fromEpochNanoseconds(-8640000000000000000000n).toJSON());

print("temporal-instant-statics:" + parts.join("|"));

262;
