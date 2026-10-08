function check(value, message) { if (!value) throw new Error(message); }
function range(action) {
  let thrown = false;
  try { action(); } catch (error) { check(error instanceof RangeError, "RangeError brand"); thrown = true; }
  check(thrown, "expected RangeError");
}
const limit = 8640000000000000000000n;
const max = new Temporal.ZonedDateTime(limit, "UTC");
const min = new Temporal.ZonedDateTime(-limit, "UTC");
const options = { largestUnit: "day", smallestUnit: "day", roundingMode: "ceil" };
check(max.until(max, options).blank && max.since(max, options).blank, "upper exact zero avoids contextual tomorrow");
check(min.until(min, options).blank && min.since(min, options).blank, "lower exact zero avoids contextual yesterday");
check(max.subtract({ nanoseconds: 1 }).epochNanoseconds === limit - 1n, "upper one nanosecond admitted");
check(min.add({ nanoseconds: 1 }).epochNanoseconds === -limit + 1n, "lower one nanosecond admitted");
range(() => max.add({ nanoseconds: 1 }));
range(() => min.subtract({ nanoseconds: 1 }));
range(() => max.add({ days: 1 }));
range(() => min.subtract({ days: 1 }));
const negative = new Temporal.ZonedDateTime(-1n, "America/New_York");
const positive = new Temporal.ZonedDateTime(1n, "America/New_York");
check(negative.until(positive, { largestUnit: "day" }).nanoseconds === 2, "same-date signed fractional difference");
check(positive.until(negative, { largestUnit: "day" }).nanoseconds === -2, "reverse signed fractional difference");
print("ok");
262;
