const DateTimeFormat = Intl.DateTimeFormat;
const method = Temporal.Instant.prototype.toLocaleString;
const instant = new Temporal.Instant(0n);
for (const options of [undefined, {}, {timeZone:'UTC'}, {timeZone:'UTC', hourCycle:'h23'}, {timeZone:'UTC', year:'numeric'}, {timeZone:'UTC', timeStyle:'long'}, {timeZone:'UTC', dateStyle:'long'}, {timeZone:'UTC', dateStyle:'full', timeStyle:'full'}]) {
  const expected = new DateTimeFormat('en-US', options).format(instant);
  if (method.call(instant, 'en-US', options) !== expected) throw 'formatter defaults or styles';
}
if (method.call(instant) !== new DateTimeFormat().format(instant)) throw 'omitted arguments';
if (method.call(instant, undefined, undefined) !== new DateTimeFormat(undefined, undefined).format(instant)) throw 'undefined arguments';

const clock = {timeZone:'UTC', hourCycle:'h23', hour:'2-digit', minute:'2-digit', second:'2-digit', fractionalSecondDigits:3};
if (method.call(new Temporal.Instant(-1n), 'en-US', clock) !== '23:59:59.999') throw 'negative nanosecond';
if (method.call(new Temporal.Instant(1n), 'en-US', clock) !== '00:00:00.000') throw 'positive nanosecond';
if (method.call(new Temporal.Instant(43200000000001n), 'en-US', {timeZone:'UTC', dayPeriod:'long'}) !== 'in the afternoon') throw 'exact noon boundary';
for (const nanoseconds of [-8640000000000000000000n, -9223372036854775809n, 9223372036854775808n, 8640000000000000000000n]) {
  const receiver = new Temporal.Instant(nanoseconds);
  if (method.call(receiver, 'en-US', clock) !== new DateTimeFormat('en-US', clock).format(receiver)) throw 'wide epoch';
}
const zonedInstant = Temporal.Instant.from('2024-03-10T07:00:00.000000001Z');
for (const timeZone of ['+05:30', 'America/New_York', 'Europe/Vienna']) {
  const options = {timeZone, dateStyle:'full', timeStyle:'full'};
  if (method.call(zonedInstant, 'en-US', options) !== new DateTimeFormat('en-US', options).format(zonedInstant)) throw 'resolved time zone';
}

const expected = new DateTimeFormat('en-US', clock).format(instant);
instant.toString = function(){throw 'receiver toString';};
instant.valueOf = function(){throw 'receiver valueOf';};
instant[Symbol.toPrimitive] = function(){throw 'receiver primitive hook';};
Object.defineProperty(instant, 'epochNanoseconds', {get(){throw 'public epoch';}});
Object.defineProperty(DateTimeFormat.prototype, 'format', {configurable:true, get(){throw 'public format getter';}});
DateTimeFormat.prototype.formatToParts = function(){throw 'public parts';};
Intl.DateTimeFormat = function(){throw 'public formatter constructor';};
Temporal.Instant = function(){throw 'public Instant constructor';};
if (method.call(instant, 'en-US', clock) !== expected) throw 'intrinsic formatting';
print('ok');
