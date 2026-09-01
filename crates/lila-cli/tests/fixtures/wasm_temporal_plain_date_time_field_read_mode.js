function observedFields(fields, reads) {
  return new Proxy(fields, {
    get: function (target, key, receiver) {
      if (typeof key === "string") reads.push(key);
      return Reflect.get(target, key, receiver);
    },
  });
}

function expectTypeError(callback, label) {
  var threw = false;
  try {
    callback();
  } catch (error) {
    if (!(error instanceof TypeError)) throw label + " wrong error";
    threw = true;
  }
  if (!threw) throw label + " missing TypeError";
}

var conversionReads = [];
var converted = Temporal.PlainDateTime.from(
  observedFields(
    {
      calendar: "iso8601",
      year: 2000,
      month: 5,
      day: 2,
      hour: 3,
    },
    conversionReads,
  ),
);
if (converted.year !== 2000 || converted.month !== 5 || converted.day !== 2 || converted.hour !== 3) {
  throw "conversion result";
}
var dateTimeFieldOrder = "day,hour,microsecond,millisecond,minute,month,monthCode,nanosecond,second,year";
if (conversionReads.join(",") !== "calendar," + dateTimeFieldOrder) {
  throw "conversion read order: " + conversionReads.join(",");
}

var receiver = new Temporal.PlainDateTime(2000, 5, 2, 3);
var withReads = [];
var updated = receiver.with(observedFields({ day: 4 }, withReads));
if (updated.day !== 4 || updated.hour !== 3) throw "with result";
if (withReads.join(",") !== "calendar,timeZone," + dateTimeFieldOrder) {
  throw "with read order: " + withReads.join(",");
}

var forbiddenCalendarGets = 0;
expectTypeError(function () {
  receiver.with({
    get calendar() {
      forbiddenCalendarGets = forbiddenCalendarGets + 1;
      return "iso8601";
    },
    day: 4,
  });
}, "with calendar");
if (forbiddenCalendarGets !== 1) throw "with read forbidden calendar";

var abruptCalendar = {};
var caughtAbruptCalendar = false;
try {
  receiver.with({
    get calendar() {
      throw abruptCalendar;
    },
    day: 4,
  });
} catch (error) {
  if (error !== abruptCalendar) throw "with calendar abrupt value";
  caughtAbruptCalendar = true;
}
if (!caughtAbruptCalendar) throw "with calendar abrupt missing";

262;
