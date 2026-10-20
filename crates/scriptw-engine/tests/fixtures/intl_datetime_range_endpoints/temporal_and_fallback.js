function check(value, message) {
  if (!value) throw new Error(message);
}
function equal(actual, expected, message) {
  check(actual === expected, message + ": " + actual + " != " + expected);
}
function periodSpace(parts) {
  for (let i = 1; i < parts.length; ++i) {
    if (parts[i].type === "dayPeriod") return parts[i - 1].value;
  }
  throw new Error("day period part missing");
}
function reconstruct(parts) {
  let result = "";
  for (const part of parts) result += part.value;
  return result;
}
const short = new Intl.DateTimeFormat("en-US", { timeStyle: "short", timeZone: "UTC" });
const space = periodSpace(short.formatRangeToParts(0, 86400));
equal(space, "\u202f", "genuine CLDR hm interval day period separator");
const formatter = new Intl.DateTimeFormat("en-US", { timeZone: "Pacific/Apia" });
const separator = "\u2009–\u2009";
const pairs = [
  [new Temporal.PlainDate(2021, 8, 4), new Temporal.PlainDate(2021, 8, 5), "8/4/2021" + separator + "8/5/2021"],
  [new Temporal.PlainDateTime(2021, 8, 4, 0, 30, 45), new Temporal.PlainDateTime(2021, 8, 4, 23, 30, 45), "8/4/2021, 12:30:45" + space + "AM" + separator + "11:30:45" + space + "PM"],
  [new Temporal.PlainTime(0, 30, 45), new Temporal.PlainTime(23, 30, 45), "12:30:45" + space + "AM" + separator + "11:30:45" + space + "PM"],
  [new Temporal.PlainMonthDay(8, 4, "gregory"), new Temporal.PlainMonthDay(8, 5, "gregory"), "8/4" + separator + "8/5"],
  [new Temporal.PlainYearMonth(2021, 8, "gregory"), new Temporal.PlainYearMonth(2021, 9, "gregory"), "8/2021" + separator + "9/2021"]
];
for (const [start, end, expected] of pairs) {
  const parts = formatter.formatRangeToParts(start, end);
  equal(reconstruct(parts), expected, "Temporal range parts reconstruction");
  equal(formatter.formatRange(start, end), expected, "Temporal range string");
  equal(formatter.formatRange(start, start), formatter.format(start), "equal range keeps single alternate");
  check(formatter.formatRangeToParts(start, start).every(part => part.source === "shared"), "equal source ownership");
}
const timeStart = pairs[2][0], timeEnd = pairs[2][1];
equal(formatter.format(timeStart), "12:30:45 AM", "single retains genuine ASCII alternative");
const dateParts = formatter.formatRangeToParts(pairs[1][0], pairs[1][1]);
for (const part of dateParts) {
  if (part.type === "year" || part.type === "month" || part.type === "day") equal(part.source, "shared", "date component sharing");
}
for (const hourCycle of ["h11", "h12"]) {
  const f = new Intl.DateTimeFormat("en-US", { hour: "numeric", minute: "numeric", second: "numeric", fractionalSecondDigits: 3, hourCycle, timeZone: "UTC" });
  const start = new Temporal.PlainTime(0, 30, 45, 123);
  const end = new Temporal.PlainTime(23, 30, 45, 123);
  const firstHour = hourCycle === "h11" ? "0" : "12";
  equal(f.formatRange(start, end), firstHour + ":30:45.123" + space + "AM" + separator + "11:30:45.123" + space + "PM", "fraction/cycle companion");
  equal(f.formatRange(end, start), "11:30:45.123" + space + "PM" + separator + firstHour + ":30:45.123" + space + "AM", "reversed endpoint identity");
  equal(periodSpace(f.formatToParts(start)), " ", "single fraction ASCII alternative");
}
const clock24 = new Intl.DateTimeFormat("en-US", { hour: "numeric", minute: "numeric", second: "numeric", hourCycle: "h23", timeZone: "UTC" });
equal(clock24.formatRange(timeStart, timeEnd), "00:30:45" + separator + "23:30:45", "24 hour domain unchanged");
print("ok");
262;
