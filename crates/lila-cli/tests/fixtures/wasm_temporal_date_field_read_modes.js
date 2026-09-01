function expectRangeError(callback, label) {
  var threw = false;
  try {
    callback();
  } catch (error) {
    if (!(error instanceof RangeError)) throw label + " wrong error";
    threw = true;
  }
  if (!threw) throw label + " missing RangeError";
}

function observedFields(counter, fields) {
  return new Proxy(fields, {
    get: function (target, key, receiver) {
      if (key === "calendar") counter.calendar = counter.calendar + 1;
      return Reflect.get(target, key, receiver);
    },
  });
}

function observedOptions(counter) {
  return {
    get overflow() {
      counter.overflow = counter.overflow + 1;
      return "constrain";
    },
  };
}

function checkCounts(counter, calendar, overflow, label) {
  if (counter.calendar !== calendar) throw label + " calendar reads";
  if (counter.overflow !== overflow) throw label + " overflow reads";
}

var dateConversion = { calendar: 0, overflow: 0 };
expectRangeError(function () {
  Temporal.PlainDate.from(
    observedFields(dateConversion, {
      calendar: "iso8601",
      year: 2000,
      monthCode: "L99M",
      day: 2,
    }),
    observedOptions(dateConversion),
  );
}, "PlainDate conversion");
checkCounts(dateConversion, 1, 1, "PlainDate conversion");

var dateWith = { calendar: 0, overflow: 0 };
var date = new Temporal.PlainDate(2000, 5, 2);
expectRangeError(function () {
  date.with(
    observedFields(dateWith, { monthCode: "L99M" }),
    observedOptions(dateWith),
  );
}, "PlainDate with");
checkCounts(dateWith, 0, 1, "PlainDate with");

var monthDayConversion = { calendar: 0, overflow: 0 };
expectRangeError(function () {
  Temporal.PlainMonthDay.from(
    observedFields(monthDayConversion, {
      calendar: "iso8601",
      monthCode: "L99M",
      day: 2,
    }),
    observedOptions(monthDayConversion),
  );
}, "PlainMonthDay conversion");
checkCounts(monthDayConversion, 1, 0, "PlainMonthDay conversion");

var monthDayWith = { calendar: 0, overflow: 0 };
var monthDay = new Temporal.PlainMonthDay(5, 2);
expectRangeError(function () {
  monthDay.with(
    observedFields(monthDayWith, { monthCode: "L99M" }),
    observedOptions(monthDayWith),
  );
}, "PlainMonthDay with");
checkCounts(monthDayWith, 1, 0, "PlainMonthDay with");

262;
