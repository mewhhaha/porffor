var foreign = __lilaCreateRealm().global;
var now = foreign.Temporal.Now;
if (Object.getPrototypeOf(now) !== foreign.Object.prototype || Object.prototype.toString.call(now) !== '[object Temporal.Now]') throw 'Now namespace Realm and tag';
var names = ['instant', 'timeZoneId', 'plainDateTimeISO', 'zonedDateTimeISO', 'plainDateISO', 'plainTimeISO'];
for (var name of names) {
  var descriptor = Object.getOwnPropertyDescriptor(now, name), method = descriptor.value;
  if (!descriptor.writable || descriptor.enumerable || !descriptor.configurable || typeof method !== 'function') throw name + ' descriptor';
  if (method.name !== name || method.length !== 0 || Object.getPrototypeOf(method) !== foreign.Function.prototype || method.hasOwnProperty('prototype')) throw name + ' method metadata';
}
var I = foreign.Temporal.Instant, Z = foreign.Temporal.ZonedDateTime;
var PD = foreign.Temporal.PlainDate, PT = foreign.Temporal.PlainTime, PDT = foreign.Temporal.PlainDateTime;
var instantMethod = now.instant, zonedMethod = now.zonedDateTimeISO;
var dateTimeMethod = now.plainDateTimeISO, dateMethod = now.plainDateISO, timeMethod = now.plainTimeISO;
foreign.Temporal.Instant = foreign.Temporal.ZonedDateTime = foreign.Temporal.PlainDate = foreign.Temporal.PlainTime = foreign.Temporal.PlainDateTime = function() { throw 'public Now result constructor'; };
var instant = instantMethod.call({});
if (Object.getPrototypeOf(instant) !== I.prototype || instant.epochNanoseconds !== 1234000000n) throw 'Now Instant Realm and shared clock';
var zoned = zonedMethod.call({}, '+01:00');
if (Object.getPrototypeOf(zoned) !== Z.prototype || zoned.epochNanoseconds !== 1234000000n || zoned.hour !== 1 || zoned.second !== 1 || zoned.millisecond !== 234 || zoned.timeZoneId !== '+01:00') throw 'Now Zoned Realm and explicit fixed zone';
var dateTime = dateTimeMethod.call({}, '+01:00');
if (Object.getPrototypeOf(dateTime) !== PDT.prototype || dateTime.year !== 1970 || dateTime.hour !== 1 || dateTime.millisecond !== 234) throw 'Now DateTime Realm';
var date = dateMethod.call({}, 'UTC');
if (Object.getPrototypeOf(date) !== PD.prototype || date.year !== 1970 || date.month !== 1 || date.day !== 1) throw 'Now Date Realm';
var time = timeMethod.call({}, '+01:00');
if (Object.getPrototypeOf(time) !== PT.prototype || time.hour !== 1 || time.second !== 1 || time.millisecond !== 234) throw 'Now Time Realm';
if (now.timeZoneId.call({}) !== 'UTC') throw 'current chosen-UTC policy';
print('ok');
262;
