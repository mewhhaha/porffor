function check(value, message) { if (!value) throw new Error(message); }
function field(parts, type) {
  for (const part of parts) if (part.type === type) return part.value;
  throw new Error('missing ' + type);
}
// Independently authored ICU4X Hebrew ISO vectors; genuine CLDR47 English names.
for (const row of [[2022, 2, 25, '5782', 'Adar I', '24'], [2022, 3, 25, '5782', 'Adar II', '22'], [2021, 2, 25, '5781', 'Adar', '13']]) {
  const formatter = new Intl.DateTimeFormat('en', { calendar: 'hebrew', timeZone: 'UTC', year: 'numeric', month: 'long', day: 'numeric' });
  const parts = formatter.formatToParts(Date.UTC(row[0], row[1] - 1, row[2]));
  check(field(parts, 'year') === row[3] && field(parts, 'month') === row[4] && field(parts, 'day') === row[5], 'genuine Hebrew standard/formatting names');
}
// Actual native vendor leap-day vector: ISO2023-09-11 -> Ethiopic2015/13/6.
const ethiopic = new Intl.DateTimeFormat('en', { calendar: 'ethiopic', timeZone: 'UTC', year: 'numeric', month: 'numeric', day: 'numeric' }).formatToParts(Date.UTC(2023, 8, 11));
check(field(ethiopic, 'year') === '2015' && field(ethiopic, 'month') === '13' && field(ethiopic, 'day') === '6', 'genuine thirteenth month reaches JS parts');
for (const locale of ['en', 'fr']) {
  for (const style of ['shortOffset', 'longOffset']) {
    const formatter = new Intl.DateTimeFormat(locale, { timeZone: '-00:30', hour: 'numeric', timeZoneName: style });
    const parts = formatter.formatToParts(0);
    const expected = locale === 'en' ? (style === 'shortOffset' ? 'GMT-0:30' : 'GMT-00:30') : (style === 'shortOffset' ? 'UTC−0:30' : 'UTC−00:30');
    check(field(parts, 'timeZoneName') === expected, 'captured genuine localized minus survives parts');
    check(parts.map(p => p.value).join('') === formatter.format(0), 'localized offset parts reconstruct formatting');
  }
}
print('ok');
262;
