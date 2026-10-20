function check(value, message) { if (!value) throw new Error(message); }
const expected = ['buddhist', 'chinese', 'coptic', 'dangi', 'ethioaa', 'ethiopic', 'gregory', 'hebrew', 'indian', 'islamic-civil', 'islamic-tbla', 'islamic-umalqura', 'iso8601', 'japanese', 'persian', 'roc'];
const actual = Intl.supportedValuesOf('calendar');
check(actual.join(',') === expected.join(','), 'exact genuine canonical calendar domain');
const locales = ['en', 'en-US', 'ar', 'ar-EG', 'zh', 'zh-Hans', 'zh-Hans-CN', 'de', 'fr', 'it', 'ja', 'ko', 'hi'];
check(Intl.DateTimeFormat.supportedLocalesOf(locales, { localeMatcher: 'lookup' }).join(',') === locales.join(','), 'all thirteen genuine locales are supported');
let associations = 0;
for (const locale of locales) {
  for (const calendar of actual) {
    const formatter = new Intl.DateTimeFormat(locale, { calendar, timeZone: 'UTC', year: 'numeric', month: 'numeric', day: 'numeric' });
    check(formatter.resolvedOptions().calendar === calendar, 'public calendar tag survives the provider wire');
    const parts = formatter.formatToParts(86400000);
    check(parts.some(p => p.type === 'month') && parts.some(p => p.type === 'day'), 'actual selected calendar formats date fields');
    check(parts.map(p => p.value).join('') === formatter.format(86400000), 'real public parts reconstruct formatting');
    associations++;
  }
}
check(associations === 208, 'all genuine associations consumed');
check(new Intl.DateTimeFormat('en-u-ca-hebrew', { calendar: 'japanese' }).resolvedOptions().calendar === 'japanese', 'explicit admitted calendar overrides the extension');
check(new Intl.DateTimeFormat('en-u-ca-islamic-civil').resolvedOptions().calendar === 'islamic-civil', 'canonical hyphenated extension crosses the AOT wire');
check(new Intl.DateTimeFormat('en', { calendar: 'islamicc' }).resolvedOptions().calendar === 'islamic-civil', 'genuine whole-value option alias canonicalizes');
check(new Intl.DateTimeFormat('en', { calendar: 'ethiopic-amete-alem' }).resolvedOptions().calendar === 'ethioaa', 'genuine Ethiopic option alias canonicalizes');
check(new Intl.DateTimeFormat('en-u-ca-islamicc').resolvedOptions().calendar === 'islamic-civil', 'genuine extension alias canonicalizes');
check(new Intl.DateTimeFormat('en', { calendar: 'islamicc-foo' }).resolvedOptions().calendar === 'gregory', 'compound keyword never uses alias-prefix replacement');
print('ok');
262;
