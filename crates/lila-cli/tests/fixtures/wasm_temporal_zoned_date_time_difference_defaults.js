// Temporal.ZonedDateTime.prototype.{until,since} default largestUnit.
//
// The pinned `defaults-to-returning-hours.js` and
// `largestunit-undefined.js` cases require an omitted, undefined or `"auto"`
// largestUnit to resolve from hour, not PlainDateTime's day fallback. The
// arithmetic body is still the PlainDateTime delegate, so this fixture also
// pins that transporting resolved settings to it does not read the user's
// getters or conversion hooks twice.

function checkDuration(value, expected, label) {
  var fields = [
    "years",
    "months",
    "weeks",
    "days",
    "hours",
    "minutes",
    "seconds",
    "milliseconds",
    "microseconds",
    "nanoseconds",
  ];
  if (!(value instanceof Temporal.Duration)) throw label + " type";
  for (var i = 0; i < fields.length; i = i + 1) {
    var field = fields[i];
    if (value[field] !== expected[i]) throw label + " " + field;
  }
}

var earlier = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC");
var later = new Temporal.ZonedDateTime(1_000_090_061_987_654_321n, "UTC");
var positive = [0, 0, 0, 0, 25, 1, 1, 987, 654, 321];

// `largestunit-undefined.js`: omitted, an empty object, explicit undefined and
// a callable options object all use the ZonedDateTime hour fallback.
checkDuration(earlier.until(later), positive, "until omitted");
checkDuration(earlier.until(later, {}), positive, "until empty");
checkDuration(
  earlier.until(later, { largestUnit: undefined }),
  positive,
  "until undefined"
);
checkDuration(earlier.until(later, function () {}), positive, "until function");
checkDuration(later.since(earlier), positive, "since omitted");
checkDuration(later.since(earlier, {}), positive, "since empty");
checkDuration(
  later.since(earlier, { largestUnit: undefined }),
  positive,
  "since undefined"
);
checkDuration(later.since(earlier, function () {}), positive, "since function");

// `defaults-to-returning-hours.js`: auto is the fallback spelling and is
// equivalent to an explicit hour largestUnit.
checkDuration(
  earlier.until(later, { largestUnit: "auto" }),
  positive,
  "until auto"
);
checkDuration(
  earlier.until(later, { largestUnit: "hours" }),
  positive,
  "until hours"
);
checkDuration(
  later.since(earlier, { largestUnit: "auto" }),
  positive,
  "since auto"
);
checkDuration(
  later.since(earlier, { largestUnit: "hours" }),
  positive,
  "since hours"
);

// `largestunit-default.js`: a smallest unit larger than hour becomes the
// default largest unit. A hard-coded hour would instead fail the unit-order
// validation.
var twoDaysLater = new Temporal.ZonedDateTime(
  1_000_172_800_000_000_000n,
  "UTC"
);
checkDuration(
  earlier.until(twoDaysLater, { smallestUnit: "day" }),
  [0, 0, 0, 2, 0, 0, 0, 0, 0, 0],
  "until day fallback"
);
checkDuration(
  twoDaysLater.since(earlier, { smallestUnit: "day" }),
  [0, 0, 0, 2, 0, 0, 0, 0, 0, 0],
  "since day fallback"
);

// The ZonedDateTime settings plan retains the user's unnegated mode because
// the selected PlainDateTime `since` delegate owns NegateRoundingMode. These
// opposite-sign ceil answers detect either zero or two negations.
checkDuration(
  later.since(earlier, { smallestUnit: "hour", roundingMode: "ceil" }),
  [0, 0, 0, 0, 26, 0, 0, 0, 0, 0],
  "since positive ceil"
);
checkDuration(
  earlier.since(later, { smallestUnit: "hour", roundingMode: "ceil" }),
  [0, 0, 0, 0, -25, 0, 0, 0, 0, 0],
  "since negative ceil"
);

// Every observable get and conversion occurs once, in GetDifferenceSettings
// order. The PlainDateTime delegate sees only the normalized primitive bag.
var log = "";
var observed = {};
Object.defineProperty(observed, "largestUnit", {
  get: function () {
    log = log + "L";
    return {
      toString: function () {
        log = log + "l";
        return "auto";
      },
    };
  },
});
Object.defineProperty(observed, "roundingIncrement", {
  get: function () {
    log = log + "I";
    return {
      valueOf: function () {
        log = log + "i";
        return 1;
      },
    };
  },
});
Object.defineProperty(observed, "roundingMode", {
  get: function () {
    log = log + "R";
    return {
      toString: function () {
        log = log + "r";
        return "trunc";
      },
    };
  },
});
Object.defineProperty(observed, "smallestUnit", {
  get: function () {
    log = log + "S";
    return {
      toString: function () {
        log = log + "s";
        return "nanosecond";
      },
    };
  },
});
checkDuration(earlier.until(later, observed), positive, "observed options");
if (log !== "LlIiRrSs") throw "option observation " + log;

print("temporal-zdt-difference-default:25h|2d|LlIiRrSs");

262;
