// End-of-month differences compare the midpoint without the day clamp: Jan
// 29th plus one month is Feb 29th, which sorts past a Feb 28th end date, so
// the month borrows instead of completing. Day counts still use the clamped
// midpoint, which is a valid date.
function until(a, b, unit) {
  return Temporal.PlainDate.from(a).until(Temporal.PlainDate.from(b), { largestUnit: unit });
}
function check(cond, label) {
  if (!cond) throw label;
}

var d1 = until("2021-01-29", "2021-02-28", "years");
check(d1.years === 0 && d1.months === 0 && d1.days === 30, "jan29-feb28");

var d2 = until("2021-01-31", "2021-02-28", "years");
check(d2.months === 0 && d2.days === 28, "jan31-feb28");

var d3 = until("2021-01-31", "2021-03-30", "years");
check(d3.months === 1 && d3.days === 30, "jan31-mar30");

var d4 = until("2021-01-28", "2021-02-28", "years");
check(d4.months === 1 && d4.days === 0, "jan28-feb28");

var s1 = Temporal.PlainDate.from("2021-02-28").since(Temporal.PlainDate.from("2020-02-29"), { largestUnit: "years" });
check(s1.years === 0 && s1.months === 11 && s1.days === 28, "since-feb29");

// No Test262 file pins until() across Feb 29th into a common-year Feb 28th;
// the uniform forward rule gives 11 months and 30 days (the since() twin
// anchors the other endpoint's day and gets 28). Pinned here so a future
// upstream expectation fails loudly instead of silently changing shape.
var u5 = until("2020-02-29", "2021-02-28", "years");
check(u5.years === 0 && u5.months === 11 && u5.days === 30, "until-feb29-forward-rule");

d1.days + d2.days + d3.days + d4.days + s1.days + u5.days;
