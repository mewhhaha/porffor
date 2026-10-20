function check(value, message) { if (!value) throw new Error(message); }
const hour = 3600000000000n;
const spring = Temporal.ZonedDateTime.from("2024-03-09T12:34:56.123456789-05:00[America/New_York]");
const calendar = spring.add({ days: 1 });
const elapsed = spring.add({ hours: 24 });
check(calendar.epochNanoseconds - spring.epochNanoseconds === 23n * hour, "spring calendar day is 23 hours");
check(elapsed.epochNanoseconds - spring.epochNanoseconds === 24n * hour, "spring elapsed day is 24 hours");
check(calendar.hour === 12 && elapsed.hour === 13, "calendar and elapsed wall times");
check(calendar.nanosecond === 789 && calendar.microsecond === 456 && calendar.millisecond === 123, "calendar addition retains fractional time");
check(spring.add({ days: 1, hours: 1 }).epochNanoseconds === elapsed.epochNanoseconds, "date inverse precedes elapsed time");
check(calendar.subtract({ days: 1 }).epochNanoseconds === spring.epochNanoseconds, "calendar subtraction across spring");
const fall = Temporal.ZonedDateTime.from("2024-11-02T12:00-04:00[America/New_York]");
check(fall.add({ days: 1 }).epochNanoseconds - fall.epochNanoseconds === 25n * hour, "fall calendar day is 25 hours");
check(fall.add({ hours: 24 }).hour === 11, "fall elapsed time differs from wall time");
const half = Temporal.ZonedDateTime.from("2024-10-05T12:00+10:30[Australia/Lord_Howe]");
check(half.add({ days: 1 }).epochNanoseconds - half.epochNanoseconds === 23n * hour + hour / 2n, "half-hour transition calendar day");
check(half.add({ hours: 24 }).hour === 12 && half.add({ hours: 24 }).minute === 30, "half-hour elapsed wall time");
print("ok");
262;
