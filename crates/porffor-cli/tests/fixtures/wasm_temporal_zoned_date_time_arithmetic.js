// Temporal.ZonedDateTime.prototype.{add,subtract,until,since,withCalendar}.
//
// These five did not exist before batch 6. `Temporal.ZonedDateTime.prototype`
// carried 18 accessors and four data methods, so `zdt.add(d)` read `undefined`
// off the prototype and the call threw `TypeError: value is not callable` —
// which is the label all 28
// intl402/Temporal/ZonedDateTime/prototype/{add,subtract,since,until}/era-boundary-*.js
// cases carried. This fixture is a cheap stand-in for four of those 28: a cold
// Test262 case in this family measured ~300 s in batch 5, so the CLI suite is
// where the feature can actually be regression-tested.
//
// The arithmetic below is DELIBERATELY the gregory era boundary rather than an
// easy 2020 date. The whole family is interesting for one reason: the gregory
// era numbering has no year zero, so "1 BCE + 1 year" must land in 1 CE and
// "1 CE - 1 year" must land in 1 BCE. ISO year 0 is bce 1. An implementation
// that did the year arithmetic on the *era year* instead of the ISO year, or
// that dropped the calendar somewhere in the ZonedDateTime -> PlainDateTime ->
// ZonedDateTime round trip, produces off-by-one answers here and correct
// answers everywhere else.
//
// Every value asserted below is copied from a real test262 file, named at the
// block that uses it, so a wrong expectation here is a wrong expectation there.
//
// BISECT ORDER if this reddens: the property-shape block first (the members are
// simply missing again), then `add` (the round trip), then `until`/`since` (the
// round trip plus the Duration family rooting), then `withCalendar` (the only
// one that does not delegate).

function expectError(errorConstructor, callback) {
  var thrown = false;
  try {
    callback();
  } catch (error) {
    if (!(error instanceof errorConstructor)) throw error;
    thrown = true;
  }
  if (!thrown) throw errorConstructor.name;
}

function show(value) {
  if (value === undefined) return "undefined";
  return "" + value;
}

var reject = { overflow: "reject" };

// ---------------------------------------------------------------------------
// Property shape. Mirrors
// built-ins/Temporal/ZonedDateTime/prototype/<name>/{prop-desc,length}.js:
// a writable, non-enumerable, configurable data property holding a function of
// length 1. (`name` is deliberately not asserted here: function-name
// installation for builtins is a separate mechanism with its own coverage, and
// a gap there would redden this fixture for a reason that is not this feature.)
// ---------------------------------------------------------------------------
var methodNames = ["withCalendar", "add", "subtract", "until", "since"];
for (var i = 0; i < methodNames.length; i++) {
  var methodName = methodNames[i];
  var descriptor = Object.getOwnPropertyDescriptor(
    Temporal.ZonedDateTime.prototype,
    methodName
  );
  if (descriptor === undefined) throw methodName + " missing";
  if (typeof descriptor.value !== "function") throw methodName + " value";
  if (descriptor.value.length !== 1) throw methodName + " length";
  if (descriptor.writable !== true) throw methodName + " writable";
  if (descriptor.enumerable !== false) throw methodName + " enumerable";
  if (descriptor.configurable !== true) throw methodName + " configurable";
  // Mirrors .../<name>/branding.js: a receiver without
  // [[InitializedTemporalZonedDateTime]] is a TypeError, and the check happens
  // before any argument is coerced.
  expectError(TypeError, function () {
    return Temporal.ZonedDateTime.prototype[methodName].call({}, undefined);
  });
}

// ---------------------------------------------------------------------------
// add / subtract across the gregory era boundary.
// Values copied verbatim from
// intl402/Temporal/ZonedDateTime/prototype/add/era-boundary-gregory.js and its
// subtract twin, which are byte-identical apart from the sign of the duration.
// ---------------------------------------------------------------------------
function zdt(era, eraYear, monthCode, day, hour, minute) {
  return Temporal.ZonedDateTime.from(
    {
      era: era,
      eraYear: eraYear,
      monthCode: monthCode,
      day: day,
      hour: hour,
      minute: minute,
      timeZone: "UTC",
      calendar: "gregory",
    },
    reject
  );
}

function checkPlain(plain, year, month, day, hour, minute, era, eraYear, label) {
  if (!(plain instanceof Temporal.PlainDateTime)) throw label + " type";
  if (plain.year !== year) throw label + " year";
  if (plain.month !== month) throw label + " month";
  if (plain.day !== day) throw label + " day";
  if (plain.hour !== hour) throw label + " hour";
  if (plain.minute !== minute) throw label + " minute";
  if (plain.second !== 0) throw label + " second";
  if (plain.era !== era) throw label + " era";
  if (plain.eraYear !== eraYear) throw label + " eraYear";
  if (plain.calendarId !== "gregory") throw label + " calendarId";
}

var oneYear = new Temporal.Duration(1);
var minusOneYear = new Temporal.Duration(-1);

// "Adding 1 year to 2 BCE lands in 1 BCE (counts backwards)".
var bce2Jun = zdt("bce", 2, "M06", 15, 12, 34);
checkPlain(bce2Jun.add(oneYear).toPlainDateTime(), 0, 6, 15, 12, 34, "bce", 1, "add bce2");
// "Adding 1 year to 1 BCE lands in 1 CE (no year zero)".
var bce1Jun = zdt("bce", 1, "M06", 15, 12, 34);
checkPlain(bce1Jun.add(oneYear).toPlainDateTime(), 1, 6, 15, 12, 34, "ce", 1, "add bce1");
// "Adding 1 year to 1 CE lands in 2 CE".
var ce1Jun = zdt("ce", 1, "M06", 15, 12, 34);
checkPlain(ce1Jun.add(oneYear).toPlainDateTime(), 2, 6, 15, 12, 34, "ce", 2, "add ce1");
// "Adding 10 years to 5 BCE lands in 6 CE (no year zero)".
var bce5Mar = zdt("bce", 5, "M03", 1, 12, 34);
checkPlain(
  bce5Mar.add(new Temporal.Duration(10)).toPlainDateTime(),
  6, 3, 1, 12, 34, "ce", 6, "add bce5"
);
// "Subtracting 1 year from 1 CE lands in 1 BCE", spelled as add(-1)...
checkPlain(ce1Jun.add(minusOneYear).toPlainDateTime(), 0, 6, 15, 12, 34, "bce", 1, "add ce1 neg");
// ...and as subtract(+1). The subtract file is the add file with the duration
// signs swapped, so these two must agree or one of the two builtins is negating
// twice.
checkPlain(ce1Jun.subtract(oneYear).toPlainDateTime(), 0, 6, 15, 12, 34, "bce", 1, "sub ce1");
// "Subtracting 1 year from 1 BCE lands in 2 BCE".
checkPlain(bce1Jun.subtract(oneYear).toPlainDateTime(), -1, 6, 15, 12, 34, "bce", 2, "sub bce1");
// "Subtracting 10 years from 5 CE lands in 6 BCE".
var ce5Mar = zdt("ce", 5, "M03", 1, 12, 34);
checkPlain(
  ce5Mar.subtract(new Temporal.Duration(10)).toPlainDateTime(),
  -5, 3, 1, 12, 34, "bce", 6, "sub ce5"
);

// The result of add/subtract is a ZonedDateTime in the receiver's own time zone
// and calendar, not a PlainDateTime and not an iso8601 value. This is what the
// era assertions above depend on, so it is asserted directly rather than left
// to be inferred from them.
var added = bce1Jun.add(oneYear);
if (!(added instanceof Temporal.ZonedDateTime)) throw "add result type";
if (added.timeZoneId !== "UTC") throw "add result timeZoneId";
if (added.calendarId !== "gregory") throw "add result calendarId";
if (added.era !== "ce") throw "add result era";
if (added.eraYear !== 1) throw "add result eraYear";
if (added.hour !== 12) throw "add result hour";
if (added.minute !== 34) throw "add result minute";
// The receiver is untouched: these are value types.
if (bce1Jun.year !== 0) throw "add receiver mutated";

// ---------------------------------------------------------------------------
// until / since across the same boundary.
// Values copied from
// intl402/Temporal/ZonedDateTime/prototype/until/era-boundary-gregory.js and
// its since twin. `until` and `since` are negations of each other, which is why
// each row below is asserted in both directions.
// ---------------------------------------------------------------------------
function checkDuration(duration, years, months, label) {
  if (!(duration instanceof Temporal.Duration)) throw label + " type";
  if (duration.years !== years) throw label + " years";
  if (duration.months !== months) throw label + " months";
  if (duration.weeks !== 0) throw label + " weeks";
  if (duration.days !== 0) throw label + " days";
  if (duration.hours !== 0) throw label + " hours";
  if (duration.minutes !== 0) throw label + " minutes";
  if (duration.seconds !== 0) throw label + " seconds";
}

var yearsOption = { largestUnit: "years" };
var monthsOption = { largestUnit: "months" };

// "1y from 1 BCE to 1 CE" / "-1y backwards from 1 BCE to 1 CE".
checkDuration(bce1Jun.until(ce1Jun, yearsOption), 1, 0, "until bce1->ce1");
checkDuration(ce1Jun.until(bce1Jun, yearsOption), -1, 0, "until ce1->bce1");
checkDuration(bce1Jun.since(ce1Jun, yearsOption), -1, 0, "since bce1<-ce1");
checkDuration(ce1Jun.since(bce1Jun, yearsOption), 1, 0, "since ce1<-bce1");
// "12mo from 1 BCE to 1 CE".
checkDuration(bce1Jun.until(ce1Jun, monthsOption), 0, 12, "until bce1->ce1 months");
checkDuration(bce1Jun.since(ce1Jun, monthsOption), 0, -12, "since bce1<-ce1 months");

// "2y 1mo from 2 BCE Dec to 2 CE Jan" / "25mo".
var bce2Dec = zdt("bce", 2, "M12", 1, 12, 34);
var ce2Jan = zdt("ce", 2, "M01", 1, 12, 34);
checkDuration(bce2Dec.until(ce2Jan, yearsOption), 2, 1, "until bce2Dec->ce2Jan");
checkDuration(bce2Dec.until(ce2Jan, monthsOption), 0, 25, "until bce2Dec->ce2Jan months");
checkDuration(ce2Jan.since(bce2Dec, yearsOption), 2, 1, "since ce2Jan<-bce2Dec");
checkDuration(ce2Jan.since(bce2Dec, monthsOption), 0, 25, "since ce2Jan<-bce2Dec months");

// "9y from 5 BCE to 5 CE (no year 0)". Nine, not ten: there is no year zero to
// count, so this is the single sharpest value in the file.
var bce5Jun = zdt("bce", 5, "M06", 15, 12, 34);
var ce5Jun = zdt("ce", 5, "M06", 15, 12, 34);
checkDuration(bce5Jun.until(ce5Jun, yearsOption), 9, 0, "until bce5->ce5");
checkDuration(bce5Jun.until(ce5Jun, monthsOption), 0, 108, "until bce5->ce5 months");
checkDuration(ce5Jun.since(bce5Jun, yearsOption), 9, 0, "since ce5<-bce5");

// A ZonedDateTime is an acceptable `other` in both spellings the corpus uses:
// the instance itself, and a property bag. Both route through
// Temporal.ZonedDateTime.from.
checkDuration(
  bce1Jun.until(
    {
      era: "ce",
      eraYear: 1,
      monthCode: "M06",
      day: 15,
      hour: 12,
      minute: 34,
      timeZone: "UTC",
      calendar: "gregory",
    },
    yearsOption
  ),
  1,
  0,
  "until property bag"
);

// A different calendar on the two sides is a RangeError, before any option is
// read. Mirrors PlainDateTime's since/different-calendars-throws.js, which this
// path shares.
expectError(RangeError, function () {
  return bce1Jun.until(bce1Jun.withCalendar("iso8601"), yearsOption);
});

// ---------------------------------------------------------------------------
// withCalendar. The one member that does not delegate: it rewrites the
// calendar slot of the record and leaves the instant alone.
//
// It is on the critical path for the `since`/`until` era-boundary files, which
// call `one.withCalendar("iso8601")` to build the ISO oracle they compare the
// weeks/days answers against (since/era-boundary-gregory.js:65).
// ---------------------------------------------------------------------------
var isoView = bce2Jun.withCalendar("iso8601");
if (!(isoView instanceof Temporal.ZonedDateTime)) throw "withCalendar type";
if (isoView.calendarId !== "iso8601") throw "withCalendar calendarId";
if (isoView.timeZoneId !== "UTC") throw "withCalendar timeZoneId";
// Same instant, re-labelled. The epoch nanoseconds are the operation's whole
// invariant.
// Compared through `show` rather than with `!==` so this line tests the
// calendar rewrite and not BigInt strict equality, which has its own coverage.
if (show(isoView.epochNanoseconds) !== show(bce2Jun.epochNanoseconds)) {
  throw "withCalendar epochNanoseconds";
}
// The ISO calendar has no eras, so both era slots go undefined while the ISO
// year stays -1.
if (isoView.year !== -1) throw "withCalendar year";
if (isoView.era !== undefined) throw "withCalendar era";
if (isoView.eraYear !== undefined) throw "withCalendar eraYear";
// The receiver is untouched.
if (bce2Jun.calendarId !== "gregory") throw "withCalendar receiver mutated";
if (bce2Jun.era !== "bce") throw "withCalendar receiver era";
// Round trip back.
if (isoView.withCalendar("gregory").era !== "bce") throw "withCalendar round trip";
// No argument is a TypeError rather than a silent switch to iso8601: an absent
// `calendar` key in a property bag defaults, an absent argument here does not.
expectError(TypeError, function () {
  return bce2Jun.withCalendar();
});

// ---------------------------------------------------------------------------
// Rendered output, so the CLI test holds the literal strings and a wrong
// rendering cannot match itself.
// ---------------------------------------------------------------------------
function renderPlain(value) {
  return (
    show(value.year) +
    "-" +
    show(value.month) +
    "-" +
    show(value.day) +
    "T" +
    show(value.hour) +
    ":" +
    show(value.minute) +
    "/" +
    show(value.era) +
    "/" +
    show(value.eraYear)
  );
}

function renderDuration(value) {
  return show(value.years) + "y" + show(value.months) + "mo";
}

var parts = [];
parts.push(renderPlain(bce2Jun.add(oneYear).toPlainDateTime()));
parts.push(renderPlain(bce1Jun.add(oneYear).toPlainDateTime()));
parts.push(renderPlain(ce1Jun.subtract(oneYear).toPlainDateTime()));
parts.push(renderPlain(ce5Mar.subtract(new Temporal.Duration(10)).toPlainDateTime()));
parts.push(renderDuration(bce5Jun.until(ce5Jun, yearsOption)));
parts.push(renderDuration(bce5Jun.until(ce5Jun, monthsOption)));
parts.push(renderDuration(bce2Dec.until(ce2Jan, yearsOption)));
parts.push(renderDuration(ce1Jun.since(bce1Jun, yearsOption)));
parts.push(show(isoView.calendarId) + "/" + show(isoView.year) + "/" + show(isoView.era));

print("temporal-zdt-arith:" + parts.join("|"));

262;
