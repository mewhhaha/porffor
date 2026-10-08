// Authored T22 named-zone control; staged and unexecuted.
function check(value, message) { if (!value) throw new Error(message); }
function expectThrow(kind, operation, message) {
  var caught = false;
  try { operation(); } catch (error) { caught = error instanceof kind; }
  check(caught, message);
}

var historical = Temporal.ZonedDateTime.from({ year: 1970, month: 1, day: 1, hour: 12, timeZone: "Africa/Monrovia" });
check(historical.offset === "-00:44:30", "historical second offset");
for (var policy of ["ignore", "prefer"]) {
  var ignored = historical.with({ day: 2, offset: "-00:45" }, { offset: policy });
  check(ignored.epochNanoseconds === 132270000000000n && ignored.hour === 12 && ignored.second === 0, "with does not fuzzy-match minute offset: " + policy);
}
var used = historical.with({ day: 2, offset: "-00:45" }, { offset: "use" });
check(used.epochNanoseconds === 132300000000000n && used.minute === 0 && used.second === 30, "use takes exact supplied offset then projects actual zone");
expectThrow(RangeError, function () { historical.with({ day: 2, offset: "-00:45" }, { offset: "reject" }); }, "reject requires exact offset match");
var base = Temporal.ZonedDateTime.from("2020-11-01T12:00-08:00[America/Los_Angeles]");
var first = base.with({ hour: 1, minute: 30 }, { offset: "ignore", disambiguation: "earlier" });
var second = base.with({ hour: 1, minute: 30 }, { offset: "ignore", disambiguation: "later" });
check(second.epochNanoseconds - first.epochNanoseconds === 3600000000000n, "retained fold disambiguation selects distinct candidates");
check(base.with({ hour: 1, minute: 30 }, { offset: "prefer", disambiguation: "earlier" }).epochNanoseconds === second.epochNanoseconds, "prefer exact receiver offset before disambiguation");
expectThrow(RangeError, function () { base.with({ hour: 1, minute: 30 }, { offset: "ignore", disambiguation: "reject" }); }, "reject fold");
var gapBase = Temporal.ZonedDateTime.from("2020-03-08T12:00-07:00[America/Los_Angeles]");
var gapEarlier = gapBase.with({ hour: 2, minute: 30 }, { offset: "ignore", disambiguation: "earlier" });
var gapLater = gapBase.with({ hour: 2, minute: 30 }, { offset: "ignore", disambiguation: "later" });
check(gapEarlier.hour === 1 && gapLater.hour === 3 && gapLater.epochNanoseconds - gapEarlier.epochNanoseconds === 3600000000000n, "gap endpoint proof feeds balanced inverse requery");

print("ok");
262;
