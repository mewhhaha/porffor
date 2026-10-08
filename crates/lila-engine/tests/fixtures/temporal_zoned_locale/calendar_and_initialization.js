function check(value, message) { if (!value) throw new Error(message); }
const epoch = 1735213600321000000n;
const iso = new Temporal.ZonedDateTime(epoch, 'UTC');
const gregory = new Temporal.ZonedDateTime(epoch, 'UTC', 'gregory');
const buddhist = new Temporal.ZonedDateTime(epoch, 'UTC', 'buddhist');
const options = { year: 'numeric', month: 'long', day: 'numeric' };
const formatter = new Intl.DateTimeFormat('en-US-u-ca-gregory', { ...options, timeZone: 'UTC' });
const expected = formatter.format(1735213600321);
check(formatter.resolvedOptions().calendar === 'gregory', 'matching case uses an available formatter calendar');
check(new Intl.DateTimeFormat('en-US-u-ca-iso8601').resolvedOptions().calendar === 'iso8601', 'mismatch case uses a different available formatter calendar');
check(iso.toLocaleString('en-US-u-ca-gregory', options) === expected, 'ISO receiver admits the locale-selected calendar');
check(gregory.toLocaleString('en-US-u-ca-gregory', options) === expected, 'actual matching receiver calendar survives');
let mismatch;
try { gregory.toLocaleString('en-US-u-ca-iso8601', options); } catch (error) { mismatch = error; }
check(mismatch && mismatch.constructor === RangeError, 'non-ISO mismatched calendar rejects');
const trace = [];
let lateMismatch;
try {
  gregory.toLocaleString('en-US-u-ca-iso8601', {
    get timeZone() { trace.push('zone'); return undefined; },
    get dateStyle() { trace.push('dateStyle'); return undefined; },
    get timeStyle() { trace.push('timeStyle'); return undefined; }
  });
} catch (error) { lateMismatch = error; }
check(lateMismatch && lateMismatch.constructor === RangeError, 'mismatch remains RangeError after initialization');
check(trace.join(',') === 'zone,dateStyle,timeStyle', 'calendar compatibility is tested after all format options');
const marker = {};
let earlyAbrupt;
try { gregory.toLocaleString('en-US-u-ca-iso8601', { get timeStyle() { throw marker; } }); } catch (error) { earlyAbrupt = error; }
check(earlyAbrupt === marker, 'last getter abrupt completion takes precedence over later calendar mismatch');
check(gregory.toLocaleString('en-US-u-ca-iso8601', { ...options, calendar: 'gregory' }) === expected, 'explicit compatible calendar remains effective');
const buddhistFormatter = new Intl.DateTimeFormat('en-US-u-ca-buddhist', { ...options, timeZone: 'UTC' });
check(buddhistFormatter.resolvedOptions().calendar === 'buddhist', 'actual Buddhist profile is available');
check(buddhist.toLocaleString('en-US-u-ca-buddhist', options) === buddhistFormatter.format(1735213600321), 'matching Buddhist receiver is formatted');
const roc = new Temporal.ZonedDateTime(epoch, 'UTC', 'roc');
const rocFormatter = new Intl.DateTimeFormat('en-US-u-ca-roc', { ...options, timeZone: 'UTC' });
check(rocFormatter.resolvedOptions().calendar === 'roc', 'genuine ROC profile is available');
check(roc.toLocaleString('en-US-u-ca-roc', options) === rocFormatter.format(1735213600321), 'matching ROC receiver is formatted');
const unsupportedFormatter = new Intl.DateTimeFormat('en-US-u-ca-islamic', { ...options, timeZone: 'UTC' });
check(unsupportedFormatter.resolvedOptions().calendar === 'gregory', 'unsupported valid Intl calendar resolves to the profile default');
let unsupportedMismatch;
try { roc.toLocaleString('en-US-u-ca-islamic', options); } catch (error) { unsupportedMismatch = error; }
check(unsupportedMismatch && unsupportedMismatch.constructor === RangeError, 'Temporal admission does not imply remaining Intl calendar availability');
print('ok');
262;
