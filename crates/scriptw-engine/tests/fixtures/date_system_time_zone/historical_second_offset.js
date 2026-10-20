function check(value, label) { if (!value) throw label; }
var epoch = -2208988800000, date = new Date(epoch);
check(date.getHours() === 0 && date.getMinutes() === 9 && date.getSeconds() === 21, 'historical forward seconds');
check(date.getTimezoneOffset() === -561 / 60, 'fractional minute offset');
check(new Date(1900, 0, 1, 0, 9, 21).getTime() === epoch, 'historical seconds inverse');
check(Date.parse('1900-01-01T00:09:21') === epoch, 'historical missing-zone parse');
check(date.toDateString() === 'Mon Jan 01 1900', 'local date string');
check(date.toTimeString() === '00:09:21 GMT+0009 (Europe/Paris, +00:09:21)', 'HHMM truncates seconds and exact suffix retains them');
check(date.toString() === 'Mon Jan 01 1900 00:09:21 GMT+0009 (Europe/Paris, +00:09:21)', 'owned complete display');
check(Date.parse(date.toString()) === epoch, 'historical display round trip');
print('ok');
262;
