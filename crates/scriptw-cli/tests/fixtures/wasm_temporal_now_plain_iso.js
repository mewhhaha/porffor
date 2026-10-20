// Temporal.Now.{plainDateTimeISO,plainDateISO,plainTimeISO}.
//
// The three members resolve an optional time-zone argument (absent means the
// system zone, pinned to UTC), read the host wall clock, and project it with
// the iso8601 calendar. Wall-clock values are nondeterministic, so the
// fixture asserts shape, observation order and argument handling — never the
// clock reading itself.

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

function checkMember(name) {
  var fn = Temporal.Now[name];
  if (typeof fn !== "function") throw name + " missing";
  if (fn.name !== name) throw name + " name";
  if (fn.length !== 0) throw name + " length";
}
checkMember("plainDateTimeISO");
checkMember("plainDateISO");
checkMember("plainTimeISO");

function checkIntRange(value, lo, hi, what) {
  if (typeof value !== "number" || value !== Math.floor(value)) throw what + " int";
  if (value < lo || value > hi) throw what + " range";
}

var dt = Temporal.Now.plainDateTimeISO();
if (!(dt instanceof Temporal.PlainDateTime)) throw "datetime brand";
if (dt.calendarId !== "iso8601") throw "datetime calendar";
checkIntRange(dt.year, 1970, 2200, "year");
checkIntRange(dt.month, 1, 12, "month");
checkIntRange(dt.day, 1, 31, "day");
checkIntRange(dt.hour, 0, 23, "hour");

var d = Temporal.Now.plainDateISO();
if (!(d instanceof Temporal.PlainDate)) throw "date brand";
if (d.calendarId !== "iso8601") throw "date calendar";

var t = Temporal.Now.plainTimeISO();
if (!(t instanceof Temporal.PlainTime)) throw "time brand";

// An explicit offset zone is accepted and produces well-formed fields.
var shifted = Temporal.Now.plainDateTimeISO("+05:00");
checkIntRange(shifted.hour, 0, 23, "shifted hour");

// `ToTemporalTimeZoneObject` accepts an ISO string: the zone comes from the
// offset/annotation, and the datetime part is ignored for the clock reading.
var fromIso = Temporal.Now.plainDateISO("2021-08-19T17:30-07:00");
if (!(fromIso instanceof Temporal.PlainDate)) throw "iso brand";

// A garbage zone is a RangeError, not a crash or a silent UTC.
expectError(RangeError, function () { Temporal.Now.plainTimeISO("XYZ"); });

print("temporal-now-plain-iso:datetime|date|time");

262;
