// Japanese calendar: Gregorian months with ISO year numbering and seven
// eras. Imperial eras resolve purely from (era, eraYear); the accessor then
// reports whichever era the resulting date falls in. Dates before Meiji 6
// (1873) report `ce`/`bce`.
var cal = "japanese";

var r = Temporal.PlainDate.from({ era: "reiwa", eraYear: 2, month: 1, day: 1, calendar: cal });
if (r.toString() !== "2020-01-01[u-ca=japanese]") throw "reiwa resolve";
if (r.era !== "reiwa" || r.eraYear !== 2) throw "reiwa accessor";

var h = Temporal.PlainDate.from({ era: "heisei", eraYear: 31, monthCode: "M05", day: 1, calendar: cal });
if (h.toString() !== "2019-05-01[u-ca=japanese]") throw "heisei31 resolve";
if (h.era !== "reiwa" || h.eraYear !== 1) throw "heisei31 crossover";

var rb = Temporal.PlainDate.from({ era: "reiwa", eraYear: 1, monthCode: "M04", day: 30, calendar: cal });
if (rb.era !== "heisei" || rb.eraYear !== 31) throw "reiwa1 crossover";

var s = Temporal.PlainDate.from({ era: "showa", eraYear: 64, monthCode: "M01", day: 8, calendar: cal });
if (s.era !== "heisei" || s.eraYear !== 1) throw "showa64 crossover";

var t = Temporal.PlainDate.from({ era: "taisho", eraYear: 1, monthCode: "M07", day: 29, calendar: cal });
if (t.era !== "meiji" || t.eraYear !== 45) throw "taisho1 crossover";

var m = Temporal.PlainDate.from({ era: "meiji", eraYear: 1, monthCode: "M10", day: 23, calendar: cal });
if (m.toString() !== "1868-10-23[u-ca=japanese]") throw "meiji1 resolve";
if (m.era !== "ce" || m.eraYear !== 1868) throw "meiji1 pre-1873";

var ce = Temporal.PlainDate.from({ era: "ce", eraYear: 1873, monthCode: "M01", day: 1, calendar: cal });
if (ce.era !== "meiji" || ce.eraYear !== 6) throw "ce1873 meiji6";

var pm = Temporal.PlainDate.from({ calendar: "japanese", era: "ce", eraYear: 1800, month: 6, day: 1 });
if (pm.era !== "ce" || pm.eraYear !== 1800) throw "pre-meiji";

var threw = false;
try {
  Temporal.PlainDate.from({ era: "roc", eraYear: 1, month: 1, day: 1, calendar: cal });
} catch (error) {
  if (!(error instanceof RangeError)) throw error;
  threw = true;
}
if (!threw) throw "foreign era accepted";

var dt = Temporal.PlainDateTime.from({ era: "reiwa", eraYear: 2, month: 1, day: 1, hour: 3, calendar: cal });
if (dt.era !== "reiwa" || dt.eraYear !== 2) throw "dateTime";

var ym = Temporal.PlainYearMonth.from({ era: "heisei", eraYear: 31, month: 4, calendar: cal });
if (ym.era !== "heisei" || ym.eraYear !== 31) throw "yearMonth";

ce.eraYear;
