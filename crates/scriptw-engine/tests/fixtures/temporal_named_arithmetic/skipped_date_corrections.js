function check(value, message) { if (!value) throw new Error(message); }
const start = Temporal.ZonedDateTime.from("2011-12-29T12:00-10:00[Pacific/Apia]");
const one = start.add({ days: 1 });
const two = start.add({ days: 2 });
check(one.epochNanoseconds === two.epochNanoseconds, "compatible skipped date maps to next date");
check(one.year === 2011 && one.month === 12 && one.day === 31 && one.hour === 12, "actual interpreted date after skip");
const complete = start.until(one, { largestUnit: "day" });
check(complete.days === 2 && complete.hours === 0, "raw difference counts actual calendar dates");
const before = one.subtract({ minutes: 30 });
const correction = start.until(before, { largestUnit: "day" });
check(correction.days === 0 && correction.hours === 23 && correction.minutes === 30, "two compatible date probes before skipped boundary");
check(start.add(correction).epochNanoseconds === before.epochNanoseconds, "bounded correction reconstructs endpoint");
const reverse = before.until(start, { largestUnit: "day" });
check(reverse.days === -1 && reverse.hours === -23 && reverse.minutes === -30, "reverse correction retains skipped calendar leg");
check(before.add(reverse).epochNanoseconds === start.epochNanoseconds, "reverse correction reconstructs endpoint");
print("ok");
262;
