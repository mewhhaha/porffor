// ROC (Minguo) calendar: proleptic Gregorian months with year = ISO - 1911.
// ROC 1 is ISO 1912; `roc` counts forward and `broc` counts backwards.
var d = Temporal.PlainDate.from({ year: 113, month: 7, day: 2, calendar: "roc" });
if (d.calendarId !== "roc") throw "calendarId";
if (d.year !== 113) throw "year";
if (d.monthCode !== "M07") throw "monthCode";
if (d.era !== "roc") throw "era";
if (d.eraYear !== 113) throw "eraYear";
if (d.toString() !== "2024-07-02[u-ca=roc]") throw "toString";

var e = Temporal.PlainDate.from({ era: "broc", eraYear: 1, month: 1, day: 1, calendar: "roc" });
if (e.year !== 0) throw "broc year";
if (e.era !== "broc") throw "broc era";
if (e.eraYear !== 1) throw "broc eraYear";
if (e.toString() !== "1911-01-01[u-ca=roc]") throw "broc toString";

var f = Temporal.PlainDate.from({ era: "roc", eraYear: 113, month: 7, day: 2, calendar: "roc" });
if (f.toString() !== "2024-07-02[u-ca=roc]") throw "roc era resolve";

var g = d.withCalendar("iso8601");
if (g.calendarId !== "iso8601" || g.toString() !== "2024-07-02") throw "withCalendar";

var h = Temporal.PlainDate.from("2024-07-02[u-ca=roc]");
if (h.calendarId !== "roc" || h.year !== 113) throw "annotated parse";

var threw = false;
try {
  Temporal.PlainDate.from({ era: "meiji", eraYear: 1, month: 1, day: 1, calendar: "roc" });
} catch (error) {
  if (!(error instanceof RangeError)) throw error;
  threw = true;
}
if (!threw) throw "foreign era accepted";

var dt = Temporal.PlainDateTime.from({ year: 113, month: 7, day: 2, hour: 3, calendar: "roc" });
if (dt.year !== 113 || dt.era !== "roc") throw "dateTime";

var ym = Temporal.PlainYearMonth.from({ year: 113, month: 7, calendar: "roc" });
if (ym.year !== 113 || ym.era !== "roc") throw "yearMonth";

var md = Temporal.PlainMonthDay.from({ monthCode: "M07", day: 2, calendar: "roc" });
if (md.calendarId !== "roc" || md.toString() !== "1972-07-02[u-ca=roc]") throw "monthDay";

113;
