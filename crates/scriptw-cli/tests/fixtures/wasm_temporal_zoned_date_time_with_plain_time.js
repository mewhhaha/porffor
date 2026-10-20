// Temporal.ZonedDateTime.prototype.withPlainTime.
//
// Brand-checks the receiver, converts the argument with `ToTemporalTime`
// (`undefined`/absent means midnight), keeps the receiver's wall-clock date,
// and reinterprets the combined fields in the receiver's zone through the same
// fixed-zone epoch tail `with` uses.
//
// Rendered values are printed as well as compared, so the CLI test holds the
// literal strings and a wrong rendering cannot match itself.

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

var withPlainTime = Temporal.ZonedDateTime.prototype.withPlainTime;
if (typeof withPlainTime !== "function") throw "withPlainTime missing";
if (withPlainTime.name !== "withPlainTime") throw "withPlainTime name";
if (withPlainTime.length !== 0) throw "withPlainTime length";

var zdt = Temporal.ZonedDateTime.from("2015-12-07T03:24:30.000003500[-08:00]");

function render(v) {
  return v.toString();
}

var fromBag = zdt.withPlainTime({ hour: 10 });
if (render(fromBag) !== "2015-12-07T10:00:00-08:00[-08:00]") throw "bag";

var fromTime = zdt.withPlainTime(new Temporal.PlainTime(11, 22));
if (render(fromTime) !== "2015-12-07T11:22:00-08:00[-08:00]") throw "time";

var fromString = zdt.withPlainTime("12:34");
if (render(fromString) !== "2015-12-07T12:34:00-08:00[-08:00]") throw "string";

// Absent and undefined both mean midnight.
var implicit = zdt.withPlainTime();
if (render(implicit) !== "2015-12-07T00:00:00-08:00[-08:00]") throw "implicit";
var explicit = zdt.withPlainTime(undefined);
if (render(explicit) !== "2015-12-07T00:00:00-08:00[-08:00]") throw "explicit";

// A date-only string throws; it never means implicit midnight.
expectError(RangeError, function () { zdt.withPlainTime("2019-10-01"); });
// A brand violation throws before the argument is converted.
expectError(TypeError, function () { withPlainTime.call({}, { hour: 1 }); });
// Subclassing is ignored: the result carries the intrinsic prototype even
// when the receiver is a subclass instance.
class Sub extends Temporal.ZonedDateTime {}
var sub = new Sub(0n, "UTC");
var subResult = sub.withPlainTime("05:43:21.123456789");
if (Object.getPrototypeOf(subResult) !== Temporal.ZonedDateTime.prototype) {
  throw "subclassing";
}
if (subResult.epochNanoseconds !== 20601_123_456_789n) throw "subclassing epoch";

print("temporal-zdt-withplaintime:" + render(fromBag) + "|" + render(fromTime) + "|" + render(fromString));
print("temporal-zdt-withplaintime-midnight:" + render(implicit) + "|" + render(explicit));

262;
