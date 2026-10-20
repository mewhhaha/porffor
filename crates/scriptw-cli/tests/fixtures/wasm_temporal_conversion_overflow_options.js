function check(condition, label) {
  if (!condition) throw label;
}

var overflowReads = 0;
var observedOptions = {
  get overflow() {
    overflowReads++;
    return "constrain";
  },
};

function rejectOverflowRead(value, label) {
  Object.defineProperty(value, "overflow", {
    get: function () {
      throw label;
    },
  });
  return value;
}

var date = new Temporal.PlainDate(2024, 5, 20);
var laterDate = rejectOverflowRead(
  new Temporal.PlainDate(2024, 5, 22),
  "PlainDate omitted overflow",
);
check(
  Temporal.PlainDate.from(date, observedOptions).day === 20,
  "PlainDate.from",
);
check(Temporal.PlainDate.compare(date, laterDate) === -1, "PlainDate.compare");
check(!date.equals(laterDate), "PlainDate.equals");
check(date.until(laterDate).days === 2, "PlainDate.until");

var yearMonth = new Temporal.PlainYearMonth(2024, 5);
var laterYearMonth = rejectOverflowRead(
  new Temporal.PlainYearMonth(2024, 7),
  "PlainYearMonth omitted overflow",
);
check(
  Temporal.PlainYearMonth.from(yearMonth, observedOptions).month === 5,
  "PlainYearMonth.from",
);
check(
  Temporal.PlainYearMonth.compare(yearMonth, laterYearMonth) === -1,
  "PlainYearMonth.compare",
);
check(!yearMonth.equals(laterYearMonth), "PlainYearMonth.equals");
check(yearMonth.until(laterYearMonth).months === 2, "PlainYearMonth.until");

var time = new Temporal.PlainTime(10, 30);
var laterTime = rejectOverflowRead(
  new Temporal.PlainTime(12, 30),
  "PlainTime omitted overflow",
);
check(
  Temporal.PlainTime.from(time, observedOptions).hour === 10,
  "PlainTime.from",
);
check(Temporal.PlainTime.compare(time, laterTime) === -1, "PlainTime.compare");
check(!time.equals(laterTime), "PlainTime.equals");
check(time.until(laterTime).hours === 2, "PlainTime.until");
check(date.toPlainDateTime(laterTime).hour === 12, "PlainDate.toPlainDateTime");

var dateTime = new Temporal.PlainDateTime(2024, 5, 20, 10, 30);
var laterDateTime = rejectOverflowRead(
  new Temporal.PlainDateTime(2024, 5, 22, 12, 30),
  "PlainDateTime omitted overflow",
);
check(
  Temporal.PlainDateTime.from(dateTime, observedOptions).hour === 10,
  "PlainDateTime.from",
);
check(
  Temporal.PlainDateTime.compare(dateTime, laterDateTime) === -1,
  "PlainDateTime.compare",
);
check(!dateTime.equals(laterDateTime), "PlainDateTime.equals");
check(dateTime.until(laterDateTime).days === 2, "PlainDateTime.until");
check(
  dateTime.withPlainTime(laterTime).hour === 12,
  "PlainDateTime.withPlainTime",
);

var monthDay = new Temporal.PlainMonthDay(5, 20);
var laterMonthDay = rejectOverflowRead(
  new Temporal.PlainMonthDay(5, 22),
  "PlainMonthDay omitted overflow",
);
check(
  Temporal.PlainMonthDay.from(monthDay, observedOptions).day === 20,
  "PlainMonthDay.from",
);
check(!monthDay.equals(laterMonthDay), "PlainMonthDay.equals");

check(overflowReads === 5, "from overflow read count");

262;
