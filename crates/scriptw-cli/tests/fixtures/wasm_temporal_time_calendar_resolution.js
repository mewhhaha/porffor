// A time string has two deliberately different calendar consumers.
// `Temporal.PlainTime` ignores a syntactically valid annotation value, while
// `ToTemporalCalendarIdentifier` (used by `withCalendar`) resolves it.

function expectRangeError(callback) {
  var threw = false;
  try {
    callback();
  } catch (error) {
    if (!(error instanceof RangeError)) throw error;
    threw = true;
  }
  if (!threw) throw "missing RangeError";
}

var time = Temporal.PlainTime.from("T11:30[u-ca=notacal]");
if (time.hour !== 11 || time.minute !== 30) throw "PlainTime ignored calendar";

var date = Temporal.PlainDate.from("2000-05-02");
if (date.withCalendar("T11:30").calendarId !== "iso8601") {
  throw "time default calendar";
}
if (date.withCalendar("T11:30[u-ca=gregorian]").calendarId !== "gregory") {
  throw "time canonical calendar";
}
expectRangeError(function () {
  date.withCalendar("T11:30[u-ca=notacal]");
});

262;
