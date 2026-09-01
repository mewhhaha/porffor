function check(condition, label) {
  if (!condition) throw label;
}

var date = new Temporal.PlainDate(2024, 5, 20);
var twoDays = new Temporal.Duration(0, 0, 0, 2);
var dateAdded = date.add(twoDays);
var dateSubtracted = date.subtract(twoDays);
check(
  dateAdded.year === 2024 && dateAdded.month === 5 && dateAdded.day === 22,
  "PlainDate add",
);
check(
  dateSubtracted.year === 2024 && dateSubtracted.month === 5 &&
    dateSubtracted.day === 18,
  "PlainDate subtract",
);

var yearMonth = new Temporal.PlainYearMonth(2024, 5);
var twoMonths = new Temporal.Duration(0, 2);
var yearMonthAdded = yearMonth.add(twoMonths);
var yearMonthSubtracted = yearMonth.subtract(twoMonths);
check(
  yearMonthAdded.year === 2024 && yearMonthAdded.month === 7,
  "PlainYearMonth add",
);
check(
  yearMonthSubtracted.year === 2024 && yearMonthSubtracted.month === 3,
  "PlainYearMonth subtract",
);

var time = new Temporal.PlainTime(10, 30);
var twoHours = new Temporal.Duration(0, 0, 0, 0, 2);
var timeAdded = time.add(twoHours);
var timeSubtracted = time.subtract(twoHours);
check(timeAdded.hour === 12 && timeAdded.minute === 30, "PlainTime add");
check(
  timeSubtracted.hour === 8 && timeSubtracted.minute === 30,
  "PlainTime subtract",
);

var dateTime = new Temporal.PlainDateTime(2024, 5, 20, 10, 30);
var dayAndTwoHours = new Temporal.Duration(0, 0, 0, 1, 2);
var dateTimeAdded = dateTime.add(dayAndTwoHours);
var dateTimeSubtracted = dateTime.subtract(dayAndTwoHours);
check(
  dateTimeAdded.year === 2024 &&
    dateTimeAdded.month === 5 &&
    dateTimeAdded.day === 21 &&
    dateTimeAdded.hour === 12 &&
    dateTimeAdded.minute === 30,
  "PlainDateTime add",
);
check(
  dateTimeSubtracted.year === 2024 &&
    dateTimeSubtracted.month === 5 &&
    dateTimeSubtracted.day === 19 &&
    dateTimeSubtracted.hour === 8 &&
    dateTimeSubtracted.minute === 30,
  "PlainDateTime subtract",
);

262;
