function check(value, message) { if (!value) throw new Error(message); }
const epoch = 86400000000000n;
const buddhist = new Temporal.ZonedDateTime(epoch, 'UTC', 'buddhist');
const iso = new Temporal.ZonedDateTime(epoch, 'UTC');
const options = { year: 'numeric', month: 'long', day: 'numeric' };
const f = new Intl.DateTimeFormat('en-US-u-ca-buddhist', { ...options, timeZone: 'UTC' });
check(buddhist.toLocaleString('en-US-u-ca-buddhist', options) === f.format(86400000), 'matching Buddhist receiver');
check(iso.toLocaleString('en-US-u-ca-buddhist', options) === f.format(86400000), 'ISO exemption');
const trace = [];
let caught;
try { buddhist.toLocaleString('en-US-u-ca-gregory', { get timeZone() { trace.push('zone'); }, get dateStyle() { trace.push('date'); }, get timeStyle() { trace.push('time'); } }); }
catch (error) { caught = error; }
check(caught && caught.constructor === RangeError, 'actual mismatched calendar');
check(trace.join(',') === 'zone,date,time', 'calendar error follows initialization');
const marker = {};
try { buddhist.toLocaleString('en-US-u-ca-gregory', { get timeStyle() { throw marker; } }); throw new Error('missing abrupt'); }
catch (error) { check(error === marker, 'last getter wins over calendar mismatch'); }
check(buddhist.toLocaleString('en-US-u-ca-gregory', { ...options, calendar: 'buddhist' }) === f.format(86400000), 'explicit matching override');
print('ok');
262;
