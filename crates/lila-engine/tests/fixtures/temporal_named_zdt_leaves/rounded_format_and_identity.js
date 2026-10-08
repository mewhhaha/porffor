// Authored T22 named-zone control; staged and unexecuted.
function check(value, message) { if (!value) throw new Error(message); }
function expectThrow(kind, operation, message) {
  var caught = false;
  try { operation(); } catch (error) { caught = error instanceof kind; }
  check(caught, message);
}

var before = Temporal.ZonedDateTime.from("2000-04-02T01:59:59.999999999-08:00[America/Vancouver]");
check(before.toString({ fractionalSecondDigits: 8, roundingMode: "halfExpand" }) === "2000-04-02T03:00:00.00000000-07:00[America/Vancouver]", "format projects rounded epoch offset");
var historical = new Temporal.ZonedDateTime(0n, "Africa/Monrovia");
check(historical.offset === "-00:44:30", "getter retains exact historical seconds");
check(historical.toString() === "1969-12-31T23:15:30-00:45[Africa/Monrovia]", "serialization rounds negative offset to minute halfExpand");
check(new Temporal.ZonedDateTime(-1n, "UTC").toString() === "1969-12-31T23:59:59.999999999+00:00[UTC]", "negative fraction uses Euclidean floor epoch");
var calcutta = new Temporal.ZonedDateTime(0n, "Asia/Calcutta");
var kolkata = new Temporal.ZonedDateTime(0n, "Asia/Kolkata");
check(calcutta.equals(kolkata) && calcutta.timeZoneId === "Asia/Calcutta", "identity uses PrimaryIdentifier while annotation retains Identifier");
check(!calcutta.equals(new Temporal.ZonedDateTime(0n, "Asia/Colombo")), "equal offset does not imply equal zone");
var gregory = calcutta.withCalendar("gregory");
check(gregory.calendarId === "gregory" && gregory.epochNanoseconds === calcutta.epochNanoseconds && gregory.timeZoneId === calcutta.timeZoneId, "withCalendar retains actual epoch and zone");
var reads = [];
var options = new Proxy({}, { get: function (target, key) { reads.push(key); return undefined; } });
before.toString(options);
check(reads.join(",") === "calendarName,fractionalSecondDigits,offset,roundingMode,smallestUnit,timeZoneName", "serialization option order before rounded projection");
var never = new Proxy({}, { get: function () { throw new Error("toJSON argument must not be read"); } });
check(before.toJSON(never) === before.toString(), "toJSON uses undefined options");
for (var zone of ["Europe/Paris", "America/Vancouver", "Africa/Monrovia", "UTC"]) {
  var z = new Temporal.ZonedDateTime(-1n, zone);
  check(z.toString().indexOf("[" + zone + "]") > 0 && z.toInstant().epochNanoseconds === -1n, "interleaved snapshots retain independent zones");
}

print("ok");
262;
