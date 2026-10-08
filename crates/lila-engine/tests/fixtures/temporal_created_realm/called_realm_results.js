var foreign = __lilaCreateRealm().global;
var I = foreign.Temporal.Instant, Z = foreign.Temporal.ZonedDateTime;
var PD = foreign.Temporal.PlainDate, PT = foreign.Temporal.PlainTime;
var PDT = foreign.Temporal.PlainDateTime, MD = foreign.Temporal.PlainMonthDay;
var YM = foreign.Temporal.PlainYearMonth, D = foreign.Temporal.Duration;
var instant = new Temporal.Instant(0n), zoned = new Temporal.ZonedDateTime(0n, 'UTC');
var date = new Temporal.PlainDate(2000, 5, 2), time = new Temporal.PlainTime(3, 4, 5);
var dateTime = new Temporal.PlainDateTime(2000, 5, 2, 3, 4, 5);
var monthDay = new Temporal.PlainMonthDay(5, 2), yearMonth = new Temporal.PlainYearMonth(2000, 5);
var duration = new Temporal.Duration(0, 0, 0, 2);
var constructors = [I, Z, PD, PT, PDT, MD, YM, D];
var names = ['Instant', 'ZonedDateTime', 'PlainDate', 'PlainTime', 'PlainDateTime', 'PlainMonthDay', 'PlainYearMonth', 'Duration'];
for (var name of names) foreign.Temporal[name] = function() { throw 'public constructor replacement'; };
function check(result, constructor, key, expected, label) {
  if (Object.getPrototypeOf(result) !== constructor.prototype || result[key] !== expected) throw label;
}
var originals = [instant, zoned, date, time, dateTime, monthDay, yearMonth, duration];
var keys = ['epochNanoseconds', 'epochNanoseconds', 'day', 'hour', 'day', 'day', 'month', 'days'];
var values = [0n, 0n, 2, 3, 2, 2, 5, 2];
for (var index = 0; index < constructors.length; index++) {
  var from = constructors[index].from;
  var result = from(originals[index]);
  if (result === originals[index]) throw names[index] + ' from copies';
  check(result, constructors[index], keys[index], values[index], names[index] + ' from Realm');
}
var zonedReads = [];
var convertedZoned = Z.from('2000-05-02T00:00[UTC]', {
  get disambiguation() {
    zonedReads.push('disambiguation');
    Temporal.PlainDate.from('2000-05-02');
    return 'compatible';
  },
  get offset() { zonedReads.push('offset'); return 'reject'; },
  get overflow() { zonedReads.push('overflow'); return 'constrain'; }
});
if (zonedReads.join(',') !== 'disambiguation,offset,overflow') throw 'zoned conversion option order';
check(convertedZoned, Z, 'day', 2, 'zoned string conversion retains called Realm across a hook');
check(Z.from({year:2000, month:5, day:2, calendar:'2000-05-02[u-ca=iso8601]', timeZone:'UTC'}), Z, 'day', 2, 'zoned bag parses its calendar before called-Realm allocation');
check(I.prototype.add.call(instant, {nanoseconds:1}), I, 'epochNanoseconds', 1n, 'Instant arithmetic');
check(Z.prototype.add.call(zoned, {seconds:1}), Z, 'epochNanoseconds', 1000000000n, 'zoned arithmetic');
check(Z.prototype.with.call(zoned, {second:1}), Z, 'second', 1, 'zoned with');
check(Z.prototype.toInstant.call(zoned), I, 'epochNanoseconds', 0n, 'zoned to Instant');
check(Z.prototype.toPlainDate.call(zoned), PD, 'day', 1, 'zoned to Date');
check(Z.prototype.toPlainTime.call(zoned), PT, 'hour', 0, 'zoned to Time');
check(Z.prototype.toPlainDateTime.call(zoned), PDT, 'year', 1970, 'zoned to DateTime');
check(PD.prototype.with.call(date, {day:3}), PD, 'day', 3, 'date with');
check(PD.prototype.add.call(date, {days:1}), PD, 'day', 3, 'date arithmetic');
check(PD.prototype.toPlainYearMonth.call(date), YM, 'month', 5, 'date to YearMonth');
check(PD.prototype.toPlainMonthDay.call(date), MD, 'day', 2, 'date to MonthDay');
check(PD.prototype.toPlainDateTime.call(date, time), PDT, 'hour', 3, 'date to DateTime');
check(PD.prototype.toZonedDateTime.call(date, 'UTC'), Z, 'epochNanoseconds', 957225600000000000n, 'date to ZonedDateTime');
check(PDT.prototype.with.call(dateTime, {hour:6}), PDT, 'hour', 6, 'DateTime with');
check(PDT.prototype.add.call(dateTime, {days:1}), PDT, 'day', 3, 'DateTime arithmetic');
check(PDT.prototype.toPlainDate.call(dateTime), PD, 'day', 2, 'DateTime to Date');
check(PDT.prototype.toPlainTime.call(dateTime), PT, 'hour', 3, 'DateTime to Time');
check(PDT.prototype.toZonedDateTime.call(dateTime, 'UTC'), Z, 'epochNanoseconds', 957236645000000000n, 'DateTime to ZonedDateTime');
check(PT.prototype.with.call(time, {hour:6}), PT, 'hour', 6, 'time with');
check(PT.prototype.add.call(time, {hours:1}), PT, 'hour', 4, 'time arithmetic');
check(YM.prototype.with.call(yearMonth, {month:6}), YM, 'month', 6, 'YearMonth with');
check(YM.prototype.add.call(yearMonth, {months:1}), YM, 'month', 6, 'YearMonth arithmetic');
check(MD.prototype.with.call(monthDay, {day:3}), MD, 'day', 3, 'MonthDay with');
check(D.prototype.negated.call(duration), D, 'days', -2, 'Duration arithmetic');
check(PD.prototype.until.call(date, new Temporal.PlainDate(2000, 5, 3)), D, 'days', 1, 'date difference Duration');
print('ok');
262;
