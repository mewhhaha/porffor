function check(value, label) { if (!value) throw label; }
var zone = Temporal.Now.timeZoneId(), date = new Date(0);
if (zone === 'America/New_York') {
  check(date.getHours() === 19 && date.getDate() === 31 && date.getTimezoneOffset() === 300, 'cached named default');
  check(Date.parse('1970-01-01T00:00:00') === 18000000, 'cached named inverse');
} else if (zone === '+01:00') {
  check(date.getHours() === 1 && date.getDate() === 1 && date.getTimezoneOffset() === -60, 'cached fixed default');
  check(Date.parse('1970-01-01T00:00:00') === -3600000, 'cached fixed inverse');
} else throw 'unrecognized configured Realm';
check(new Intl.DateTimeFormat('en', {year:'numeric'}).resolvedOptions().timeZone === zone, 'cached Intl default association');
check(Temporal.Now.instant().epochNanoseconds === 1234000000n, 'cached executing Realm clock');
print(zone);
262;
