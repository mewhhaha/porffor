// Authored T22 named-zone control; staged and unexecuted.
function check(value, message) { if (!value) throw new Error(message); }
function expectThrow(kind, operation, message) {
  var caught = false;
  try { operation(); } catch (error) { caught = error instanceof kind; }
  check(caught, message);
}

var toronto = Temporal.ZonedDateTime.from("1919-03-31T12:00-04:00[America/Toronto]");
var start = toronto.startOfDay();
check(start.hour === 0 && start.minute === 30, "cross-midnight gap starts at earliest valid local time");
check(toronto.withPlainTime().epochNanoseconds === start.epochNanoseconds, "absent time uses GetStartOfDay");
check(toronto.withPlainTime("00:00").epochNanoseconds === start.epochNanoseconds + 1800000000000n, "explicit midnight uses compatible inverse");
var fold = Temporal.ZonedDateTime.from("2020-11-01T01:30-08:00[America/Los_Angeles]");
var earlier = fold.withPlainTime("01:30");
check(earlier.epochNanoseconds === fold.epochNanoseconds - 3600000000000n && earlier.offset === "-07:00", "explicit fold time chooses compatible earlier epoch");
check(Temporal.ZonedDateTime.from("2020-10-04T12:00+11:00[Australia/Lord_Howe]").hoursInDay === 23.5, "half-hour forward day");
check(Temporal.ZonedDateTime.from("2020-04-05T12:00+10:30[Australia/Lord_Howe]").hoursInDay === 24.5, "half-hour backward day");
check(Temporal.ZonedDateTime.from("1933-01-01T12:00+07:20[Asia/Singapore]").hoursInDay === 23 + 2 / 3, "twenty-minute forward day");

print("ok");
262;
