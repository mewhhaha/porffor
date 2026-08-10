// Temporal.ZonedDateTime.prototype.era, .eraYear and .toPlainDateTime.
//
// The two era accessors answer from `emit_temporal_calendar_era_field`, the one
// emitter PlainDate, PlainDateTime and PlainYearMonth also use, so the four
// cannot disagree about where the year-0 boundary falls or which era codes
// exist.
//
// The `bce` half of this fixture is the part that earns its place. A
// ZonedDateTime derives its local ISO year through
// `emit_date_components_from_time`, which leaves every component as an f64 *bit
// pattern*; the era emitter wants a plain integer ISO year. A getter that
// forgets to reinterpret and truncate still answers "ce" for every positive
// year, because a reinterpreted positive double is a large positive integer and
// the era test is `isoYear > 0`. Only a non-positive year can see that bug,
// which is why every era assertion below is paired with its `bce` mirror.
//
// Rendered values are printed as well as compared, so the CLI test holds the
// literal strings and a wrong rendering cannot match itself.

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
// Property shape. Mirrors built-ins/Temporal/ZonedDateTime/prototype/{era,
// eraYear}/prop-desc.js: an accessor with a getter, no setter, not enumerable,
// configurable.
// ---------------------------------------------------------------------------
var accessorNames = ["era", "eraYear"];
for (var i = 0; i < accessorNames.length; i++) {
  var name = accessorNames[i];
  var descriptor = Object.getOwnPropertyDescriptor(
    Temporal.ZonedDateTime.prototype,
    name
  );
  if (descriptor === undefined) throw name + " missing";
  if (typeof descriptor.get !== "function") throw name + " get";
  if (descriptor.get.length !== 0) throw name + " get length";
  if (descriptor.set !== undefined) throw name + " set";
  if (descriptor.enumerable !== false) throw name + " enumerable";
  if (descriptor.configurable !== true) throw name + " configurable";
}

// Mirrors .../{era,eraYear}/branding.js: the getter is brand-checked, so every
// receiver without [[InitializedTemporalZonedDateTime]] is a TypeError —
// including the constructor and the prototype the getter lives on.
var eraGetter = Object.getOwnPropertyDescriptor(
  Temporal.ZonedDateTime.prototype,
  "era"
).get;
var eraYearGetter = Object.getOwnPropertyDescriptor(
  Temporal.ZonedDateTime.prototype,
  "eraYear"
).get;
// `Symbol()` is in this list because branding.js lists it: a symbol is the one
// primitive whose ToObject conversion is not a plain wrapper the brand check
// might accidentally accept, and leaving it out was the only divergence from
// the test262 file this block mirrors.
var badReceivers = [
  undefined,
  null,
  true,
  "",
  1,
  {},
  Symbol(),
  Temporal.ZonedDateTime,
  Temporal.ZonedDateTime.prototype,
];
for (var j = 0; j < badReceivers.length; j++) {
  var receiver = badReceivers[j];
  expectError(TypeError, function () {
    return eraGetter.call(receiver);
  });
  expectError(TypeError, function () {
    return eraYearGetter.call(receiver);
  });
}

if (typeof Temporal.ZonedDateTime.prototype.toPlainDateTime !== "function") {
  throw "toPlainDateTime missing";
}
if (Temporal.ZonedDateTime.prototype.toPlainDateTime.length !== 0) {
  throw "toPlainDateTime length";
}
expectError(TypeError, function () {
  return Temporal.ZonedDateTime.prototype.toPlainDateTime.call({});
});

// ---------------------------------------------------------------------------
// The two intl402 basic.js assertions, spelled out.
// ---------------------------------------------------------------------------
var epoch = new Temporal.ZonedDateTime(0n, "UTC", "gregory");
if (epoch.era !== "ce") throw "epoch era";
if (epoch.eraYear !== 1970) throw "epoch eraYear";
if (epoch.year !== 1970) throw "epoch year";

// The ISO 8601 calendar has no eras, so both slots are `undefined` rather than
// a borrowed ce/bce. This is the pair that already "passed" before the
// accessors existed, for the wrong reason — the property was simply absent.
var iso = new Temporal.ZonedDateTime(0n, "UTC");
if (iso.era !== undefined) throw "iso era";
if (iso.eraYear !== undefined) throw "iso eraYear";
if (iso.calendarId !== "iso8601") throw "iso calendarId";

// ---------------------------------------------------------------------------
// The year-0 boundary, from both sides. `bce` counts backwards from ISO year 1,
// so ISO 0 is bce 1 and ISO -1 is bce 2, and there is no year zero in the era
// numbering.
// ---------------------------------------------------------------------------
var ce1 = Temporal.ZonedDateTime.from(
  {
    era: "ce",
    eraYear: 1,
    monthCode: "M01",
    day: 1,
    timeZone: "UTC",
    calendar: "gregory",
  },
  reject
);
if (ce1.year !== 1) throw "ce1 year";
if (ce1.era !== "ce") throw "ce1 era";
if (ce1.eraYear !== 1) throw "ce1 eraYear";

var bce1 = Temporal.ZonedDateTime.from(
  {
    era: "bce",
    eraYear: 1,
    monthCode: "M01",
    day: 1,
    timeZone: "UTC",
    calendar: "gregory",
  },
  reject
);
if (bce1.year !== 0) throw "bce1 year";
if (bce1.era !== "bce") throw "bce1 era";
if (bce1.eraYear !== 1) throw "bce1 eraYear";

var bce2 = Temporal.ZonedDateTime.from(
  {
    era: "bce",
    eraYear: 2,
    monthCode: "M06",
    day: 15,
    hour: 12,
    minute: 34,
    timeZone: "UTC",
    calendar: "gregory",
  },
  reject
);
if (bce2.year !== -1) throw "bce2 year";
if (bce2.era !== "bce") throw "bce2 era";
if (bce2.eraYear !== 2) throw "bce2 eraYear";

// ---------------------------------------------------------------------------
// toPlainDateTime carries the calendar, which is the whole reason the
// era-boundary corpus can assert anything: TemporalHelpers.assertPlainDateTime
// reads `era`/`eraYear`/`calendarId` off a PlainDateTime, never off the zoned
// value. Dropping the calendar here would hand back an iso8601 PlainDateTime
// that reports no era at all, and every one of those files would fail on the
// era line rather than on the arithmetic.
// ---------------------------------------------------------------------------
var bce2Plain = bce2.toPlainDateTime();
if (!(bce2Plain instanceof Temporal.PlainDateTime)) throw "toPlainDateTime type";
if (bce2Plain.calendarId !== "gregory") throw "toPlainDateTime calendarId";
if (bce2Plain.year !== -1) throw "toPlainDateTime year";
if (bce2Plain.month !== 6) throw "toPlainDateTime month";
if (bce2Plain.monthCode !== "M06") throw "toPlainDateTime monthCode";
if (bce2Plain.day !== 15) throw "toPlainDateTime day";
if (bce2Plain.hour !== 12) throw "toPlainDateTime hour";
if (bce2Plain.minute !== 34) throw "toPlainDateTime minute";
if (bce2Plain.second !== 0) throw "toPlainDateTime second";
if (bce2Plain.era !== "bce") throw "toPlainDateTime era";
if (bce2Plain.eraYear !== 2) throw "toPlainDateTime eraYear";

// An iso8601 receiver hands back an iso8601 PlainDateTime, not a gregory one.
if (iso.toPlainDateTime().calendarId !== "iso8601") throw "iso toPlainDateTime calendarId";
if (iso.toPlainDateTime().era !== undefined) throw "iso toPlainDateTime era";

// Sub-millisecond components survive the split. The nanosecond remainder is the
// one component that never passes through the f64 date-component path, so it is
// the one a blanket "reinterpret and truncate" pass would corrupt.
var nanos = new Temporal.ZonedDateTime(217175010123456789n, "UTC", "gregory");
var nanosPlain = nanos.toPlainDateTime();
if (nanosPlain.year !== 1976) throw "nanos year";
if (nanosPlain.month !== 11) throw "nanos month";
if (nanosPlain.day !== 18) throw "nanos day";
if (nanosPlain.hour !== 14) throw "nanos hour";
if (nanosPlain.minute !== 23) throw "nanos minute";
if (nanosPlain.second !== 30) throw "nanos second";
if (nanosPlain.millisecond !== 123) throw "nanos millisecond";
if (nanosPlain.microsecond !== 456) throw "nanos microsecond";
if (nanosPlain.nanosecond !== 789) throw "nanos nanosecond";

// A pre-epoch instant exercises the negative-remainder correction that turns a
// truncating i64 division into a floor. -13849764999999999n renders as
// 1969-07-24T16:50:35.000000001Z, so every sub-second component is a different
// shape from the positive case above.
var preEpoch = new Temporal.ZonedDateTime(-13849764999999999n, "UTC", "gregory");
var preEpochPlain = preEpoch.toPlainDateTime();
if (preEpochPlain.year !== 1969) throw "pre-epoch year";
if (preEpochPlain.month !== 7) throw "pre-epoch month";
if (preEpochPlain.day !== 24) throw "pre-epoch day";
if (preEpochPlain.hour !== 16) throw "pre-epoch hour";
if (preEpochPlain.minute !== 50) throw "pre-epoch minute";
if (preEpochPlain.second !== 35) throw "pre-epoch second";
if (preEpochPlain.millisecond !== 0) throw "pre-epoch millisecond";
if (preEpochPlain.microsecond !== 0) throw "pre-epoch microsecond";
if (preEpochPlain.nanosecond !== 1) throw "pre-epoch nanosecond";
if (preEpochPlain.era !== "ce") throw "pre-epoch era";
if (preEpochPlain.eraYear !== 1969) throw "pre-epoch eraYear";

// A non-UTC fixed offset shifts the *local* date, and therefore can move the
// era across the boundary for one and the same instant. -62135596800000000000n
// is 0001-01-01T00:00:00Z; at -12:00 that same instant is 0000-12-31T12:00,
// i.e. ce 1 read one way and bce 1 read the other. This is what pins the era to
// the local ISO year rather than to the epoch value.
//
// The offset is spelled `-12:00` rather than `Etc/GMT+12` on purpose:
// `Temporal.ZonedDateTime` accepts `UTC` and `UTCOffset` identifiers, while the
// `Etc/GMT±N` table is `Intl.DateTimeFormat`'s.
//
// BISECT THIS BLOCK FIRST if the CLI test reddens. It is the only place in the
// fixture that leans on two paths outside the era feature, neither of which
// has other wasm-aot coverage here: -62135596800000000000n is about -6.21e19,
// far outside i64, so it takes the two-limb heap-BigInt path
// (`emit_temporal_heap_bigint_millisecond_quotient`, `builtins/temporal.rs`)
// rather than the inline arm, and `withTimeZone("-12:00")` goes through
// `emit_temporal_fixed_time_zone_offset_seconds`. A regression in either
// presents here as an era failure and is not one.
var atCeBoundary = new Temporal.ZonedDateTime(
  -62135596800000000000n,
  "UTC",
  "gregory"
);
if (atCeBoundary.year !== 1) throw "boundary utc year";
if (atCeBoundary.month !== 1) throw "boundary utc month";
if (atCeBoundary.day !== 1) throw "boundary utc day";
if (atCeBoundary.era !== "ce") throw "boundary utc era";
if (atCeBoundary.eraYear !== 1) throw "boundary utc eraYear";
var shifted = atCeBoundary.withTimeZone("-12:00");
if (shifted.year !== 0) throw "boundary shifted year";
if (shifted.month !== 12) throw "boundary shifted month";
if (shifted.day !== 31) throw "boundary shifted day";
if (shifted.hour !== 12) throw "boundary shifted hour";
if (shifted.era !== "bce") throw "boundary shifted era";
if (shifted.eraYear !== 1) throw "boundary shifted eraYear";
var shiftedPlain = shifted.toPlainDateTime();
if (shiftedPlain.year !== 0) throw "boundary shifted plain year";
if (shiftedPlain.era !== "bce") throw "boundary shifted plain era";
if (shiftedPlain.eraYear !== 1) throw "boundary shifted plain eraYear";

function render(value) {
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
    ":" +
    show(value.second) +
    "." +
    show(value.millisecond) +
    "." +
    show(value.microsecond) +
    "." +
    show(value.nanosecond) +
    "/" +
    show(value.era) +
    "/" +
    show(value.eraYear)
  );
}

var parts = [];
parts.push(show(epoch.era));
parts.push(show(epoch.eraYear));
parts.push(show(iso.era));
parts.push(show(iso.eraYear));
parts.push(show(ce1.year) + "/" + show(ce1.era) + "/" + show(ce1.eraYear));
parts.push(show(bce1.year) + "/" + show(bce1.era) + "/" + show(bce1.eraYear));
parts.push(show(bce2.year) + "/" + show(bce2.era) + "/" + show(bce2.eraYear));
parts.push(render(bce2Plain));
parts.push(render(nanosPlain));
parts.push(render(preEpochPlain));
parts.push(render(shiftedPlain));

print("temporal-zdt-era:" + parts.join("|"));

262;
