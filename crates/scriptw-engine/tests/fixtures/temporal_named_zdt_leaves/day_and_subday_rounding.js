// Authored T22 named-zone control; staged and unexecuted.
function check(value, message) { if (!value) throw new Error(message); }
function expectThrow(kind, operation, message) {
  var caught = false;
  try { operation(); } catch (error) { caught = error instanceof kind; }
  check(caught, message);
}

function roundDay(source) { return Temporal.ZonedDateTime.from(source).round({ smallestUnit: "day", roundingMode: "halfExpand" }); }
var autumnDown = roundDay("2000-10-29T11:29:59-08:00[America/Vancouver]");
var autumnUp = roundDay("2000-10-29T11:30:01-08:00[America/Vancouver]");
check(autumnDown.day === 29 && autumnDown.hour === 0 && autumnDown.offset === "-07:00", "25-hour midpoint rounds down by elapsed progress");
check(autumnUp.day === 30 && autumnUp.hour === 0 && autumnUp.offset === "-08:00", "25-hour midpoint rounds up by elapsed progress");
var springDown = roundDay("2000-04-02T12:29:59-07:00[America/Vancouver]");
var springUp = roundDay("2000-04-02T12:30:01-07:00[America/Vancouver]");
check(springDown.day === 2 && springDown.hour === 0 && springDown.offset === "-08:00", "23-hour midpoint rounds down");
check(springUp.day === 3 && springUp.hour === 0 && springUp.offset === "-07:00", "23-hour midpoint rounds up");
var beforeGap = Temporal.ZonedDateTime.from("2000-04-02T01:59:59.999999999-08:00[America/Vancouver]");
var rounded = beforeGap.round({ smallestUnit: "microsecond", roundingMode: "halfExpand" });
check(rounded.hour === 3 && rounded.offset === "-07:00" && rounded.epochNanoseconds === beforeGap.epochNanoseconds + 1n, "subday rounding reinterprets balanced clock with original offset prefer");
// The 00:01 backward change revisits yesterday after tomorrow already began.
// Day rounding clamps to the first tomorrow start minus 1ns before rounding.
var revisited = Temporal.ZonedDateTime.from("1987-10-24T23:30-04:00[America/Goose_Bay]");
check(revisited.epochNanoseconds === 562131000000000000n && revisited.hoursInDay === 24, "revisited date retains its first-start day span");
var firstTomorrow = 562129200000000000n;
check(revisited.round({ smallestUnit: "day", roundingMode: "ceil" }).epochNanoseconds === firstTomorrow, "ceil clips progress before the next date's earliest start");
check(revisited.round({ smallestUnit: "day", roundingMode: "halfExpand" }).epochNanoseconds === firstTomorrow, "nearest day clips a post-midnight backward revisit");
check(revisited.round({ smallestUnit: "day", roundingMode: "floor" }).epochNanoseconds === 562042800000000000n, "floor returns the first start of the revisited date");
var latest = new Temporal.ZonedDateTime(8640000000000000000000n, "+23:59");
check(latest.round("nanosecond").epochNanoseconds === latest.epochNanoseconds, "nanosecond clone precedes local ISO range checks");
expectThrow(RangeError, function () { return latest.hoursInDay; }, "tomorrow start is range-checked even for an unselected endpoint");
expectThrow(RangeError, function () { latest.round({ smallestUnit: "day", roundingMode: "floor" }); }, "day rounding evaluates the range-checked tomorrow start before selecting today");

print("ok");
262;
