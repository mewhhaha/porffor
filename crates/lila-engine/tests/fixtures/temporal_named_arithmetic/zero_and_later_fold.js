function check(value, message) { if (!value) throw new Error(message); }
const later = Temporal.ZonedDateTime.from("2024-11-03T01:30:00.000000007-05:00[America/New_York]");
for (const duration of [{ days: 0 }, { hours: 0 }, { nanoseconds: 0 }]) {
  check(later.add(duration).epochNanoseconds === later.epochNanoseconds, "zero addition preserves later fold");
  check(later.subtract(duration).epochNanoseconds === later.epochNanoseconds, "zero subtraction preserves later fold");
}
check(later.add({ nanoseconds: 1 }).epochNanoseconds === later.epochNanoseconds + 1n, "tiny elapsed addition preserves fold");
check(later.subtract({ nanoseconds: 8 }).epochNanoseconds === later.epochNanoseconds - 8n, "fractional borrow preserves fold");
check(later.subtract({ days: 1 }).epochNanoseconds === later.epochNanoseconds - 25n * 3600000000000n, "date subtraction resolves previous offset");
const negative = new Temporal.ZonedDateTime(-1n, "America/New_York");
check(negative.add({ nanoseconds: 1 }).epochNanoseconds === 0n, "negative Euclidean remainder carry");
check(negative.subtract({ nanoseconds: 999999999 }).epochNanoseconds === -1000000000n, "negative whole-second borrow");
print("ok");
262;
