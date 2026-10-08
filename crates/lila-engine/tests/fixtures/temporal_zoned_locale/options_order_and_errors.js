function check(value, message) { if (!value) throw new Error(message); }
const zdt = new Temporal.ZonedDateTime(0n, 'Europe/Vienna');
const trace = [];
const locales = [];
Object.defineProperty(locales, '0', { get() { trace.push('locales'); return 'en-US'; } });
locales.length = 1;
const names = ['localeMatcher', 'calendar', 'numberingSystem', 'hour12', 'hourCycle', 'timeZone', 'weekday', 'era', 'year', 'month', 'day', 'dayPeriod', 'hour', 'minute', 'second', 'fractionalSecondDigits', 'timeZoneName', 'formatMatcher', 'dateStyle', 'timeStyle'];
const options = {};
for (const name of names) Object.defineProperty(options, name, { get() { trace.push(name); return undefined; } });
zdt.toLocaleString(locales, options);
check(trace.join(',') === ['locales', ...names].join(','), 'all original Intl option getters occur once in specified order');
for (const supplied of ['Europe/Vienna', 'UTC', null, { toString() { throw new Error('forbidden timeZone coercion'); } }]) {
  const seen = [];
  let caught;
  try {
    zdt.toLocaleString('en-US', {
      get timeZone() { seen.push('timeZone'); return supplied; },
      get weekday() { seen.push('weekday'); return 'long'; }
    });
  } catch (error) { caught = error; }
  check(caught && caught.constructor === TypeError, 'any supplied timeZone is a TypeError before ToString');
  check(seen.join(',') === 'timeZone', 'forbidden timeZone prevents later component getters');
}
const marker = {};
let abrupt;
try { zdt.toLocaleString('en-US', { get timeZone() { throw marker; } }); } catch (error) { abrupt = error; }
check(abrupt === marker, 'timeZone getter abrupt identity survives');
const foreign = __lilaCreateRealm().global;
const intrinsicTypeError = foreign.TypeError;
const borrowed = foreign.Temporal.ZonedDateTime.prototype.toLocaleString;
let realmError;
try {
  borrowed.call(zdt, { toString() { return 'en-US'; }, length: 0 }, {
    get timeZone() {
      foreign.TypeError = function ReplacedTypeError() {};
      new Temporal.ZonedDateTime(0n, 'UTC').toLocaleString('en-US');
      return 'UTC';
    }
  });
} catch (error) { realmError = error; }
check(realmError && Object.getPrototypeOf(realmError) === intrinsicTypeError.prototype, 'forbidden option uses borrowed builtin defining Realm after reentrant formatting');
print('ok');
262;
