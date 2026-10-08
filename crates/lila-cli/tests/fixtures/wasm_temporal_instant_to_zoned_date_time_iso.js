// Temporal.Instant.prototype.toZonedDateTimeISO.
//
// Brand-checks the receiver, rejects a missing time zone with a TypeError,
// resolves the zone through `ToTemporalTimeZoneObject` (an ISO string
// contributes its offset/annotation), and allocates the `ZonedDateTime` with
// the receiver's epoch nanoseconds and the `iso8601` calendar.

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

var toZonedDateTimeISO = Temporal.Instant.prototype.toZonedDateTimeISO;
if (typeof toZonedDateTimeISO !== "function") throw "missing";
if (toZonedDateTimeISO.name !== "toZonedDateTimeISO") throw "name";
if (toZonedDateTimeISO.length !== 1) throw "length";

var inst = new Temporal.Instant(1_000_000_000_000_000_000n);

var utc = inst.toZonedDateTimeISO("UTC");
if (!(utc instanceof Temporal.ZonedDateTime)) throw "brand";
if (utc.epochNanoseconds !== 1_000_000_000_000_000_000n) throw "epoch";
if (utc.timeZoneId !== "UTC") throw "zone";
if (utc.calendarId !== "iso8601") throw "calendar";

var offset = inst.toZonedDateTimeISO("-05:00");
if (offset.timeZoneId !== "-05:00") throw "offset zone";

// An ISO string contributes its offset; a missing zone is a TypeError and a
// non-string primitive is a TypeError, never a silent coercion.
var fromIso = inst.toZonedDateTimeISO("2021-08-19T17:30-07:00");
if (fromIso.timeZoneId !== "-07:00") throw "iso zone";
expectError(TypeError, function () { inst.toZonedDateTimeISO(); });
expectError(TypeError, function () { inst.toZonedDateTimeISO(undefined); });
expectError(TypeError, function () { inst.toZonedDateTimeISO(1); });
expectError(TypeError, function () { inst.toZonedDateTimeISO({}); });
expectError(RangeError, function () { inst.toZonedDateTimeISO("XYZ"); });
expectError(TypeError, function () { toZonedDateTimeISO.call({}, "UTC"); });

print("temporal-instant-tozoneddatetimeiso:UTC|-05:00|-07:00");

262;
