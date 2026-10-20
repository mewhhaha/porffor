// Temporal.Instant.prototype.toString options and toJSON.
//
// `toString` reads `fractionalSecondDigits`, `roundingMode`, `smallestUnit`
// and `timeZone` in spec order, rounds the epoch to the implied quantum,
// shifts it into the zone, and renders; `smallestUnit` overrides
// `fractionalSecondDigits` for rounding and display. `toJSON` is the same
// core with `undefined` options and never reads its argument.

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

var instant = new Temporal.Instant(999_999_960_040_000_000n); // 2001-09-09T01:46:00.04Z

if (instant.toString() !== "2001-09-09T01:46:00.04Z") throw "auto";
if (instant.toString({}) !== "2001-09-09T01:46:00.04Z") throw "empty";
if (instant.toString({ fractionalSecondDigits: 0 }) !== "2001-09-09T01:46:00Z") throw "digits0";
if (instant.toString({ fractionalSecondDigits: 3 }) !== "2001-09-09T01:46:00.040Z") throw "digits3";
if (instant.toString({ smallestUnit: "minute" }) !== "2001-09-09T01:46Z") throw "minute";
if (instant.toString({ smallestUnit: "second" }) !== "2001-09-09T01:46:00Z") throw "second";
if (instant.toString({ smallestUnit: "millisecond" }) !== "2001-09-09T01:46:00.040Z") throw "ms";

// `smallestUnit` overrides `fractionalSecondDigits`.
if (instant.toString({ smallestUnit: "second", fractionalSecondDigits: 5 }) !== "2001-09-09T01:46:00Z") {
  throw "override";
}

// Rounding applies to the epoch, with carry across minutes.
var late = new Temporal.Instant(999_999_960_040_000_000n + 59_959_999_999n);
if (late.toString({ smallestUnit: "minute" }) !== "2001-09-09T01:46Z") throw "trunc-default";
if (late.toString({ smallestUnit: "minute", roundingMode: "halfExpand" }) !== "2001-09-09T01:47Z") {
  throw "carry";
}

// A time zone shifts the wall clock and changes the suffix to the offset.
if (instant.toString({ timeZone: "UTC" }) !== "2001-09-09T01:46:00.04+00:00") throw "utc";
if (instant.toString({ timeZone: "+01:00" }) !== "2001-09-09T02:46:00.04+01:00") throw "offset";

// `toJSON` ignores its argument, even a throwing Proxy.
if (instant.toJSON() !== "2001-09-09T01:46:00.04Z") throw "json";
var bomb = new Proxy({}, { get: function () { throw "toJSON read its argument"; } });
if (instant.toJSON(bomb) !== "2001-09-09T01:46:00.04Z") throw "json arg";
if (Temporal.Instant.prototype.toJSON === Temporal.Instant.prototype.toString) throw "identity";

expectError(RangeError, function () { instant.toString({ smallestUnit: "hour" }); });
expectError(RangeError, function () { instant.toString({ fractionalSecondDigits: 10 }); });
expectError(TypeError, function () { instant.toString(1); });

print("temporal-instant-tostring:options|rounding|zone|json");

262;
