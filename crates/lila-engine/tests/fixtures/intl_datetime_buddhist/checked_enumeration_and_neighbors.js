function check(value, message) { if (!value) throw new Error(message); }
check(Intl.supportedValuesOf('calendar').join(',') === 'buddhist,chinese,coptic,dangi,ethioaa,ethiopic,gregory,hebrew,indian,islamic-civil,islamic-tbla,islamic-umalqura,iso8601,japanese,persian,roc', 'only actual sorted calendars');
for (const calendar of Intl.supportedValuesOf('calendar')) {
  check(new Intl.DateTimeFormat('en-US', { calendar }).resolvedOptions().calendar === calendar, 'no enumerated fallback');
}
const gregory = new Intl.DateTimeFormat('en-US', { calendar: 'gregory', timeZone: 'UTC', year: 'numeric' });
check(gregory.formatToParts(86400000).some(p => p.type === 'year' && p.value === '1970'), 'neighbor Gregorian');
const chinese = new Intl.DateTimeFormat('zh-u-ca-chinese', { timeZone: 'UTC', year: 'numeric' });
const parts = chinese.formatToParts(1707523200000);
check(parts.some(p => p.type === 'yearName' && p.value === '甲辰'), 'actual Chinese cyclic neighbor');
check(parts.filter(p => p.type === 'relatedYear').every(p => p.value === '2024'), 'Chinese related year');
check(!parts.some(p => p.type === 'year'), 'cyclic year keeps own classification');
print('ok');
262;
