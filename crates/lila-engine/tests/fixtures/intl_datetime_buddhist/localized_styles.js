function check(value, message) { if (!value) throw new Error(message); }
const epoch = 86400000;
for (const [locale, expected] of [['en-US', 'January 2, 2513 BE'], ['zh', '佛历2513年1月2日']]) {
  const f = new Intl.DateTimeFormat(locale, { calendar: 'buddhist', timeZone: 'UTC', dateStyle: 'long' });
  check(f.resolvedOptions().calendar === 'buddhist', 'actual calendar');
  check(f.format(epoch) === expected, 'pinned long pattern: ' + locale);
  const parts = f.formatToParts(epoch);
  check(parts.map(p => p.value).join('') === expected, 'shared pattern and parts');
  check(parts.some(p => p.type === 'year' && p.value === '2513'), 'solar year');
  check(parts.some(p => p.type === 'era' && p.value === (locale === 'zh' ? '佛历' : 'BE')), 'pinned era');
}
const arabic = new Intl.DateTimeFormat('ar-EG', { calendar: 'buddhist', timeZone: 'UTC', year: 'numeric', era: 'long' });
const parts = arabic.formatToParts(epoch);
check(parts.some(p => p.type === 'year' && p.value === '٢٥١٣'), 'actual Arabic digits');
check(parts.some(p => p.type === 'era' && p.value === 'التقويم البوذي'), 'wide Arabic era');
check(parts.map(p => p.value).join('') === arabic.format(epoch), 'Arabic join');
print('ok');
262;
