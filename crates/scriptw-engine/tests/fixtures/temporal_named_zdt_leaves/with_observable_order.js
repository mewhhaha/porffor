// Authored T22 named-zone control; staged and unexecuted.
function check(value, message) { if (!value) throw new Error(message); }
function expectThrow(kind, operation, message) {
  var caught = false;
  try { operation(); } catch (error) { caught = error instanceof kind; }
  check(caught, message);
}

var base = new Temporal.ZonedDateTime(0n, "Europe/Paris");
var reads = [];
var values = { day: 2, hour: 1, microsecond: 0, millisecond: 0, minute: 0, month: 1, monthCode: "M01", nanosecond: 0, offset: "+01:00", second: 0, year: 1970 };
var fields = new Proxy(values, { get: function (target, key) { reads.push("fields." + key); return target[key]; } });
var options = new Proxy({ disambiguation: "compatible", offset: "prefer", overflow: "constrain" }, { get: function (target, key) { reads.push("options." + key); return target[key]; } });
var changed = base.with(fields, options);
check(changed.day === 2 && changed.timeZoneId === "Europe/Paris", "ordered with result");
check(reads.join(",") === "fields.calendar,fields.timeZone,fields.day,fields.hour,fields.microsecond,fields.millisecond,fields.minute,fields.month,fields.monthCode,fields.nanosecond,fields.offset,fields.second,fields.year,options.disambiguation,options.offset,options.overflow", "partial fields and options read in spec order");
var marker = {};
var laterRead = false;
var caught;
try { base.with({ day: 2 }, { get disambiguation() { throw marker; }, get offset() { laterRead = true; return "prefer"; } }); } catch (error) { caught = error; }
check(caught === marker && !laterRead, "abrupt option identity precedes later option read");
var partialMarker = {};
var optionRead = false;
caught = undefined;
try { base.with({ get calendar() { throw partialMarker; } }, { get disambiguation() { optionRead = true; return "compatible"; } }); } catch (error) { caught = error; }
check(caught === partialMarker && !optionRead, "partial validation precedes options and zone projection");
Object.defineProperty(base, "timeZoneId", { get: function () { throw new Error("receiver zone getter must be suppressed"); } });
Object.defineProperty(base, "calendarId", { get: function () { throw new Error("receiver calendar getter must be suppressed"); } });
check(base.with({ day: 2 }).day === 2 && base.withPlainTime("05:00").hour === 5, "branded internal slots bypass receiver getters");

print("ok");
262;
