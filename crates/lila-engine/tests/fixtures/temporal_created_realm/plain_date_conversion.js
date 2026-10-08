var foreign = __lilaCreateRealm().global;
var from = foreign.Temporal.PlainDate.from;
var plainPrototype = foreign.Temporal.PlainDate.prototype;
var total = foreign.Temporal.Duration.prototype.total;
var typeErrorPrototype = foreign.TypeError.prototype;
var rangeErrorPrototype = foreign.RangeError.prototype;
var marker = {}, trace = '';
Object.defineProperty(foreign.Temporal, 'PlainDate', {
  configurable: true, get() { throw marker; }
});
foreign.TypeError = foreign.RangeError = function() { throw marker; };

var date = from({
  get calendar() { trace += 'c'; return 'iso8601'; },
  get day() { trace += 'd'; return 29; },
  get month() { trace += 'm'; return 2; },
  get monthCode() { trace += 'k'; return 'M02'; },
  get year() { trace += 'y'; Temporal.PlainDate.from('2000-01-01'); return 2024; }
}, {get overflow() { trace += 'o'; return 'reject'; }});
if (trace !== 'cdmkyo' || Object.getPrototypeOf(date) !== plainPrototype || date.year !== 2024 || date.month !== 2 || date.day !== 29) throw 'shared PlainDate conversion order/called Realm';

var branded = new Temporal.PlainDate(2000, 1, 2), reads = 0;
for (var key of ['year', 'calendar', 'overflow']) {
  Object.defineProperty(branded, key, {get() { throw marker; }});
}
var copied = from(branded, {get overflow() {
  reads++; Temporal.PlainDate.from('2001-01-01'); return undefined;
}});
if (reads !== 1 || copied === branded || Object.getPrototypeOf(copied) !== plainPrototype || copied.year !== 2000) throw 'shared branded copy/options/called Realm';
if (Temporal.PlainDate.compare(branded, copied) !== 0) throw 'Omit reads only original slots';

var caught;
try { from(1); } catch (error) { caught = error; }
if (!caught || Object.getPrototypeOf(caught) !== typeErrorPrototype) throw 'shared PlainDate conversion TypeError Realm';
trace = ''; caught = undefined;
try {
  from({
    get day() { trace += 'd'; return 1; },
    get month() { trace += 'm'; return 1; },
    get monthCode() { trace += 'k'; Temporal.PlainDate.from('2000-01-01'); return 'invalid'; },
    get year() { trace += 'y'; return 2024; }
  }, {get overflow() { trace += 'o'; return 'reject'; }});
} catch (error) { caught = error; }
if (trace !== 'dmk' || !caught || Object.getPrototypeOf(caught) !== rangeErrorPrototype) throw 'shared PlainDate conversion range/order after nested hook';
caught = undefined;
try { from(branded, {get overflow() { throw marker; }}); }
catch (error) { caught = error; }
if (caught !== marker) throw 'shared PlainDate preserves whole overflow Throw';

var duration = new Temporal.Duration(0, 0, 0, 1);
if (total.call(duration, {relativeTo: branded, unit: 'day'}) !== 1) throw 'borrowed Duration plain relative slots';
caught = undefined;
try { total.call(duration, {relativeTo: 'not-a-date', unit: 'day'}); }
catch (error) { caught = error; }
if (!caught || Object.getPrototypeOf(caught) !== rangeErrorPrototype) throw 'Duration nested PlainDate conversion called Realm';
print('ok');
262;
