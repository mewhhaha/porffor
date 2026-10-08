var foreign = __lilaCreateRealm().global;
var typeErrorPrototype = foreign.TypeError.prototype, rangeErrorPrototype = foreign.RangeError.prototype;
var getters = [
  Object.getOwnPropertyDescriptor(foreign.Temporal.Instant.prototype, 'epochNanoseconds').get,
  Object.getOwnPropertyDescriptor(foreign.Temporal.ZonedDateTime.prototype, 'epochNanoseconds').get,
  Object.getOwnPropertyDescriptor(foreign.Temporal.PlainDate.prototype, 'year').get,
  Object.getOwnPropertyDescriptor(foreign.Temporal.PlainTime.prototype, 'hour').get,
  Object.getOwnPropertyDescriptor(foreign.Temporal.PlainDateTime.prototype, 'year').get,
  Object.getOwnPropertyDescriptor(foreign.Temporal.PlainMonthDay.prototype, 'monthCode').get,
  Object.getOwnPropertyDescriptor(foreign.Temporal.PlainYearMonth.prototype, 'year').get,
  Object.getOwnPropertyDescriptor(foreign.Temporal.Duration.prototype, 'days').get
];
var timeWith = foreign.Temporal.PlainTime.prototype.with;
var durationWith = foreign.Temporal.Duration.prototype.with;
var conversion = foreign.Temporal.PlainDateTime.prototype.toZonedDateTime;
var zonedFrom = foreign.Temporal.ZonedDateTime.from;
foreign.Error = foreign.TypeError = foreign.RangeError = function() { throw 'public Error constructor must not run'; };
for (var getter of getters) {
  var caught = undefined;
  try { getter.call({}); } catch (error) { caught = error; }
  if (!caught || Object.getPrototypeOf(caught) !== typeErrorPrototype) throw 'borrowed accessor intrinsic TypeError Realm';
}
var caught, calls = 0;
try {
  timeWith.call(new Temporal.PlainTime(3), {
    get hour() { calls++; new Temporal.Instant(0n).add({nanoseconds:1}); return 24; }
  }, {overflow:'reject'});
} catch (error) { caught = error; }
if (calls !== 1 || !caught || Object.getPrototypeOf(caught) !== rangeErrorPrototype) throw 'nested hook restores RangeError Realm';
caught = undefined;
try {
  durationWith.call(new Temporal.Duration(), {
    get nanoseconds() { calls++; new Temporal.PlainDate(2000, 5, 2).add({days:1}); return Symbol('bad number'); }
  });
} catch (error) { caught = error; }
if (calls !== 2 || !caught || Object.getPrototypeOf(caught) !== typeErrorPrototype) throw 'nested hook restores TypeError Realm';
caught = undefined;
try {
  conversion.call(new Temporal.PlainDateTime(2000, 10, 29, 1, 45), 'America/Los_Angeles', {
    get disambiguation() {
      calls++;
      new Temporal.PlainDateTime(2000, 5, 2).toZonedDateTime('UTC');
      return 'reject';
    }
  });
} catch (error) { caught = error; }
if (calls !== 3 || !caught || Object.getPrototypeOf(caught) !== rangeErrorPrototype) throw 'nested inverse rejection intrinsic Realm';
var marker = {};
caught = undefined;
try { timeWith.call(new Temporal.PlainTime(), {get hour() { throw marker; }}); }
catch (error) { caught = error; }
if (caught !== marker) throw 'user abrupt completion identity';
caught = undefined;
try { zonedFrom(1); } catch (error) { caught = error; }
if (!caught || Object.getPrototypeOf(caught) !== typeErrorPrototype) throw 'shared zoned conversion intrinsic TypeError Realm';
var zonedReads = [];
caught = undefined;
try {
  zonedFrom('2000-05-02T00:00[UTC]', {
    get disambiguation() {
      zonedReads.push('disambiguation');
      Temporal.PlainDate.from('2000-05-02');
      return 'invalid';
    },
    get offset() { zonedReads.push('offset'); return 'reject'; }
  });
} catch (error) { caught = error; }
if (zonedReads.join(',') !== 'disambiguation' || !caught || Object.getPrototypeOf(caught) !== rangeErrorPrototype) throw 'shared zoned conversion retains Realm and abrupt read prefix';
caught = undefined;
try {
  zonedFrom('2000-05-02T00:00[UTC]', {
    get disambiguation() { throw marker; },
    get offset() { throw 'offset read after abrupt disambiguation'; }
  });
} catch (error) { caught = error; }
if (caught !== marker) throw 'shared zoned conversion keeps thrown marker identity';
print('ok');
262;
