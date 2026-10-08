if (Intl.ListFormat.supportedLocalesOf(['es', 'he', 'ar', 'sr', 'en-US']).join(',') !== 'es,he,en-US') {
  throw 'projected public List locale inventory';
}
if (new Intl.ListFormat('ar').resolvedOptions().locale !== 'en-US' ||
    new Intl.ListFormat('sr').resolvedOptions().locale !== 'en-US') {
  throw 'private Duration dependencies leaked into public List resolution';
}
if (new Intl.ListFormat('es').format(['A', 'B']) !== 'A y B' ||
    new Intl.ListFormat('he').format(['A', 'B']).length === 0) {
  throw 'actual projected Spanish or Hebrew List data';
}
for (const style of ['long', 'short', 'narrow']) {
  for (const type of ['conjunction', 'disjunction', 'unit']) {
    const formatter = new Intl.ListFormat('es', { style, type });
    if (formatter.resolvedOptions().locale !== 'es' ||
        formatter.format(['A', 'B', 'C']).length === 0) {
      throw 'projected marker and width closure';
    }
  }
}
const duration = new Intl.DurationFormat('sr', { style: 'long' });
if (duration.resolvedOptions().locale !== 'sr' ||
    duration.format({ hours: 1, minutes: 2 }).length === 0) {
  throw 'retained private Duration List association';
}
if (new Intl.NumberFormat('ar').resolvedOptions().locale !== 'ar' ||
    new Intl.Locale('iw-IL').toString() !== 'he-IL') {
  throw 'other selected component authorities';
}
print('intl-list-projection:ok');
262;
