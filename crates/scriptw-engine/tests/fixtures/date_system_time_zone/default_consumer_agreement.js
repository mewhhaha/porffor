function check(value, label) { if (!value) throw label; }
check(Temporal.Now.timeZoneId() === 'America/New_York', 'alias publishes primary default');
check(Date.now() === 1234 && new Date().getTime() === 1234, 'fixed clock Date consumers');
var date = new Date();
check(date.getFullYear() === 1969 && date.getMonth() === 11 && date.getDate() === 31 && date.getHours() === 19 && date.getSeconds() === 1, 'Date default projection');
check(Temporal.Now.instant().epochNanoseconds === 1234000000n, 'exact Now clock');
var zoned = Temporal.Now.zonedDateTimeISO();
check(zoned.timeZoneId === 'America/New_York' && zoned.epochNanoseconds === 1234000000n && zoned.hour === 19 && zoned.day === 31, 'Now default exact zone');
check(Temporal.Now.plainDateISO().year === 1969 && Temporal.Now.plainTimeISO().hour === 19 && Temporal.Now.plainDateTimeISO().day === 31, 'all omitted Now projections');
check(Temporal.Now.zonedDateTimeISO('+01:00').hour === 1 && Temporal.Now.plainDateISO('UTC').year === 1970, 'explicit Now zone stays separate');
var formatter = new Intl.DateTimeFormat('en-US', {hour:'2-digit',minute:'2-digit',hourCycle:'h23'});
check(formatter.resolvedOptions().timeZone === 'America/New_York', 'Intl default uses same primary');
check(formatter.format(0) === '19:00', 'Intl default formats actual epoch');
check(new Intl.DateTimeFormat('en-US', {timeZone:'UTC',hour:'2-digit',minute:'2-digit',hourCycle:'h23'}).format(0) === '00:00', 'explicit Intl zone stays separate');
check(new Date(0).toLocaleDateString('en', {year:'numeric'}) === '1969', 'Date locale default uses same selected zone');
check(new Date(0).toLocaleDateString('en', {timeZone:'UTC',year:'numeric'}) === '1970', 'explicit Date locale zone stays separate');
print('ok');
262;
