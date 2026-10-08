function check(value, message) { if (!value) throw new Error(message); }
const root = Temporal.ZonedDateTime.from("2024-03-09T12:00-05:00[America/New_York]");
const expectedEpoch = root.epochNanoseconds + 23n * 3600000000000n;
for (const name of ["epochNanoseconds", "calendarId", "timeZoneId"]) {
  Object.defineProperty(root, name, { get() { throw new Error("branded slot getter " + name); } });
}
const seen = [];
const fields = new Proxy({}, { get(target, key) {
  seen.push(key);
  if (key === "days") return { valueOf() { seen.push("days.valueOf"); return 1; } };
  return undefined;
} });
const options = { get overflow() {
  seen.push("overflow");
  return { toString() { seen.push("overflow.toString"); return "constrain"; } };
} };
const result = root.add(fields, options);
check(result.epochNanoseconds === expectedEpoch, "ordered addition result");
check(seen.join(",") === "days,days.valueOf,hours,microseconds,milliseconds,minutes,months,nanoseconds,seconds,weeks,years,overflow,overflow.toString", "duration then overflow reads exactly once");
const abrupt = {};
let reads = 0;
try {
  root.add({ get days() { throw abrupt; } }, { get overflow() { reads++; return "constrain"; } });
  throw new Error("missing duration abrupt");
} catch (error) { check(error === abrupt && reads === 0, "duration abrupt precedes options"); }
try {
  root.add({ days: 1 }, { get overflow() { throw abrupt; } });
  throw new Error("missing overflow abrupt");
} catch (error) { check(error === abrupt, "overflow abrupt identity"); }
print("ok");
262;
