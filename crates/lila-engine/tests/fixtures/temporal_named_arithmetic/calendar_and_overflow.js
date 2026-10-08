function check(value, message) { if (!value) throw new Error(message); }
function range(action) {
  let thrown = false;
  try { action(); } catch (error) { check(error instanceof RangeError, "RangeError brand"); thrown = true; }
  check(thrown, "expected RangeError");
}
const jan = Temporal.ZonedDateTime.from("2024-01-31T12:00-05:00[America/New_York]");
const feb = jan.add({ months: 1 });
check(feb.year === 2024 && feb.month === 2 && feb.day === 29 && feb.hour === 12, "constrain leap month");
range(() => jan.add({ months: 1 }, { overflow: "reject" }));
const buddhist = new Temporal.ZonedDateTime(jan.epochNanoseconds, "America/New_York", "buddhist");
const added = buddhist.add({ months: 1 });
check(added.epochNanoseconds === feb.epochNanoseconds && added.calendarId === "buddhist", "retained calendar arithmetic");
check(added.year === 2567 && added.month === 2 && added.day === 29, "retained calendar fields");
const alias = new Temporal.ZonedDateTime(jan.epochNanoseconds, "US/Eastern", "roc");
check(alias.add({ months: 1 }).epochNanoseconds === feb.epochNanoseconds, "alias arithmetic uses actual transitions");
check(alias.add({ months: 1 }).calendarId === "roc", "alias addition retains calendar");
print("ok");
262;
