function check(value, message) { if (!value) throw new Error(message); }
function range(action) {
  let thrown = false;
  try { action(); } catch (error) { check(error instanceof RangeError, "RangeError brand"); thrown = true; }
  check(thrown, "expected RangeError");
}
const start = Temporal.ZonedDateTime.from("2024-03-09T12:00-05:00[America/New_York]");
const elsewhere = new Temporal.ZonedDateTime(start.epochNanoseconds, "America/Los_Angeles");
check(start.until(elsewhere).blank, "elapsed equal epochs across zones");
range(() => start.until(elsewhere, { largestUnit: "day" }));
const alias = new Temporal.ZonedDateTime(start.epochNanoseconds, "US/Eastern");
check(start.until(alias, { largestUnit: "day" }).blank, "primary zone identity precedes zero shortcut");
const wrongCalendar = new Temporal.ZonedDateTime(start.epochNanoseconds, "America/New_York", "buddhist");
let optionsRead = false;
range(() => start.until(wrongCalendar, { get largestUnit() { optionsRead = true; return "day"; } }));
check(!optionsRead, "calendar mismatch precedes options");
const seen = [];
const options = new Proxy({}, { get(target, key) {
  seen.push(key);
  if (key === "largestUnit") return "day";
  if (key === "roundingIncrement") return 1;
  if (key === "roundingMode") return "trunc";
  if (key === "smallestUnit") return "nanosecond";
  return undefined;
} });
check(start.until(alias, options).blank, "ordered equal-epoch difference");
check(seen.join(",") === "largestUnit,roundingIncrement,roundingMode,smallestUnit", "difference settings read once in prescribed order");
const abrupt = {};
try {
  start.until(alias, { get largestUnit() { throw abrupt; } });
  throw new Error("missing options abrupt");
} catch (error) { check(error === abrupt, "difference options abrupt identity"); }
print("ok");
262;
