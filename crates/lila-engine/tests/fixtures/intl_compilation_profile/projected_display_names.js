if (Intl.DisplayNames.supportedLocalesOf(['ja', 'de', 'fr', 'en-US', 'ar', 'en']).join(',') !== 'ja,fr,en-US' ||
    Intl.DisplayNames.supportedLocalesOf(['ja-JP', 'fr-FR']).join(',') !== 'ja-JP,fr-FR' ||
    Intl.DisplayNames.supportedLocalesOf(['ar-EG', 'iw', 'sh']).length !== 0) {
  throw 'projected DisplayNames inventory and locale lookup';
}
if (new Intl.DisplayNames('de', { type: 'region' }).resolvedOptions().locale !== 'en-US' ||
    new Intl.DisplayNames([], { type: 'region' }).resolvedOptions().locale !== 'en-US') {
  throw 'projected DisplayNames mandatory fallback';
}
for (const locale of ['en-US', 'fr', 'ja']) {
  for (const style of ['long', 'short', 'narrow']) {
    for (const pair of [['language', 'fr'], ['region', 'US'], ['script', 'Hans'],
                       ['currency', 'USD'], ['calendar', 'gregory'], ['dateTimeField', 'year']]) {
      const formatter = new Intl.DisplayNames(locale, { type: pair[0], style, fallback: 'none' });
      if (formatter.resolvedOptions().locale !== locale || typeof formatter.of(pair[1]) !== 'string') {
        throw 'projected DisplayNames complete domain and width associations';
      }
    }
  }
}
for (const row of [['long', 'United States', 'year'], ['short', 'US', 'yr.'], ['narrow', 'US', 'yr']]) {
  if (new Intl.DisplayNames('en-US', { type: 'region', style: row[0] }).of('us') !== row[1] ||
      new Intl.DisplayNames('en-US', { type: 'dateTimeField', style: row[0] }).of('year') !== row[2] ||
      new Intl.DisplayNames('fr', { type: 'currency', style: row[0] }).of('USD') !== 'dollar des États-Unis') {
    throw 'projected DisplayNames actual native names';
  }
}
if (new Intl.DisplayNames('fr-FR', { type: 'region', style: 'short' }).of('US') !== 'É.-U.' ||
    new Intl.DisplayNames('ja-JP', { type: 'region' }).of('US') !== 'アメリカ合衆国' ||
    new Intl.DisplayNames('en-US', { type: 'language' }).of('IW') !== 'Hebrew' ||
    new Intl.DisplayNames('en-US', { type: 'language', languageDisplay: 'standard' }).of('en-GB') !== 'English (United Kingdom)' ||
    new Intl.DisplayNames('en-US', { type: 'language', style: 'short' }).of('en-GB') !== 'UK English') {
  throw 'selected DisplayNames aliases and language display tables';
}
const fields = new Intl.DisplayNames('fr', { type: 'dateTimeField', fallback: 'none' });
for (const field of ['era', 'year', 'quarter', 'month', 'weekOfYear', 'weekday', 'day',
                     'dayPeriod', 'hour', 'minute', 'second', 'timeZoneName']) {
  if (typeof fields.of(field) !== 'string') throw 'projected complete date-time field names';
}
if (Intl.getCanonicalLocales('iw-IL')[0] !== 'he-IL' ||
    new Intl.NumberFormat('hi').resolvedOptions().locale !== 'hi' ||
    new Intl.PluralRules('hi').resolvedOptions().locale !== 'hi' ||
    new Intl.Collator('de').resolvedOptions().locale !== 'de' ||
    new Intl.Segmenter('ja').resolvedOptions().locale !== 'ja' ||
    new Intl.DateTimeFormat('ja', { timeZone: 'UTC' }).resolvedOptions().locale !== 'ja') {
  throw 'DisplayNames projection preserves independent component authorities';
}
print('intl-displaynames-projection:ok');
262;
