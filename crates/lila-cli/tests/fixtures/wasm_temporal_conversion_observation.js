// Temporal conversion observation order: branded-argument slot fast paths,
// `with` calendar/timeZone rejection reads, `toZonedDateTime` option reads,
// and constructor time-zone string validation.
//
// `ToTemporalDate`/`ToTemporalTime` must convert `PlainDateTime` and
// `ZonedDateTime` arguments from slots without running a single getter;
// `PlainDate.prototype.with` must read `calendar`/`timeZone` with `Get`;
// `toZonedDateTime` reads `disambiguation` only; the `ZonedDateTime`
// constructor rejects a datetime string as its time zone.

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

function getterBomb(message) {
  return function () { throw message; };
}

// 1. Slot fast paths: throwing getters on the argument are never invoked.
var zoned = new Temporal.ZonedDateTime(0n, "UTC");
Object.defineProperty(zoned, "calendar", { get: getterBomb("zoned calendar read") });
Object.defineProperty(zoned, "year", { get: getterBomb("zoned year read") });
var duration = new Temporal.PlainDate(1976, 11, 18).until(zoned);
if (!(duration instanceof Temporal.Duration)) throw "until brand";

var datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);
Object.defineProperty(datetime, "hour", { get: getterBomb("datetime hour read") });
Object.defineProperty(datetime, "calendar", { get: getterBomb("datetime calendar read") });
var time = Temporal.PlainTime.from(datetime);
if (time.hour !== 12 || time.minute !== 34 || time.second !== 56) throw "from time";
if (time.millisecond !== 987 || time.microsecond !== 654 || time.nanosecond !== 321) {
  throw "from subsecond";
}
var date = new Temporal.PlainDate(2000, 5, 2).until(datetime);
if (!(date instanceof Temporal.Duration)) throw "until datetime brand";

// 2. `PlainDate.prototype.with` reads `calendar` and `timeZone` with `Get`
// before the date fields.
var reads = [];
var bag = {};
Object.defineProperty(bag, "calendar", { get: function () { reads.push("calendar"); return undefined; } });
Object.defineProperty(bag, "timeZone", { get: function () { reads.push("timeZone"); return undefined; } });
bag.day = 5;
var withResult = new Temporal.PlainDate(2000, 5, 2).with(bag);
if (withResult.day !== 5) throw "with day";
if (reads.length !== 2 || reads[0] !== "calendar" || reads[1] !== "timeZone") {
  throw "with reads: " + reads.join(",");
}
expectError(TypeError, function () {
  new Temporal.PlainDate(2000, 5, 2).with({ calendar: "iso8601" });
});

// 3. `toZonedDateTime` reads `disambiguation` but never `offset`.
var offsetRead = false;
var disambiguationRead = false;
var options = {};
Object.defineProperty(options, "offset", {
  get: function () { offsetRead = true; return undefined; },
});
Object.defineProperty(options, "disambiguation", {
  get: function () { disambiguationRead = true; return undefined; },
});
new Temporal.PlainDateTime(2000, 5, 2, 12).toZonedDateTime("UTC", options);
if (offsetRead) throw "offset read";
if (!disambiguationRead) throw "disambiguation unread";
expectError(RangeError, function () {
  new Temporal.PlainDateTime(2000, 5, 2, 12).toZonedDateTime("UTC", { disambiguation: "bogus" });
});

// 4. The constructor rejects a datetime string as its time zone, while the
// lenient entry points accept ISO strings.
expectError(RangeError, function () {
  return new Temporal.ZonedDateTime(0n, "1997-12-04T12:34[+01:00]", "iso8601");
});
var ok = new Temporal.ZonedDateTime(0n, "+01:00", "iso8601");
if (ok.timeZoneId !== "+01:00") throw "offset zone";
var lenient = Temporal.Now.zonedDateTimeISO("2021-08-19T17:30-07:00");
if (!(lenient instanceof Temporal.ZonedDateTime)) throw "lenient brand";

print("temporal-conversion-order:slots|with|options|zones");

262;
