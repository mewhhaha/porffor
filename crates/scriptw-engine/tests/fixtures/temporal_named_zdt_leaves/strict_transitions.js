// Authored T22 named-zone control; staged and unexecuted.
function check(value, message) { if (!value) throw new Error(message); }
function expectThrow(kind, operation, message) {
  var caught = false;
  try { operation(); } catch (error) { caught = error instanceof kind; }
  check(caught, message);
}

var transition = Temporal.Instant.from("2021-03-28T01:00:00Z").epochNanoseconds;
var justBefore = new Temporal.ZonedDateTime(transition - 1n, "Europe/Berlin");
var exact = new Temporal.ZonedDateTime(transition, "Europe/Berlin");
var justAfter = new Temporal.ZonedDateTime(transition + 1n, "Europe/Berlin");
check(justBefore.getTimeZoneTransition("next").epochNanoseconds === transition, "next includes transition after negative nanosecond neighbor");
check(justAfter.getTimeZoneTransition("previous").epochNanoseconds === transition, "previous includes transition before positive nanosecond neighbor");
check(exact.getTimeZoneTransition("next").epochNanoseconds > transition && exact.getTimeZoneTransition("previous").epochNanoseconds < transition, "exact query is strict in both directions");
var london = new Temporal.ZonedDateTime(0n, "Europe/London").getTimeZoneTransition("previous");
check(london.epochNanoseconds === -59004000000000000n, "abbreviation-only rule change is not an offset transition");
var first = -842918400000000000n;
var second = -842223600000000000n;
check(new Temporal.ZonedDateTime(first + 1n, "Africa/Tunis").getTimeZoneTransition("next").epochNanoseconds === second, "closely spaced transitions are not skipped");
for (var zone of ["UTC", "+05:30"]) {
  var log = [];
  var value = new Temporal.ZonedDateTime(0n, zone).getTimeZoneTransition({ get direction() { log.push("get"); return { toString: function () { log.push("string"); return "next"; } }; } });
  check(value === null && log.join(",") === "get,string", "fixed-zone absence follows direction coercion");
  expectThrow(RangeError, function () { new Temporal.ZonedDateTime(0n, zone).getTimeZoneTransition("sideways"); }, "fixed zone does not skip invalid direction");
}

print("ok");
262;
