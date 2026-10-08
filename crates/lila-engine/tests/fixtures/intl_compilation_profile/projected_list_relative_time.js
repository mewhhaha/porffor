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
if (Intl.RelativeTimeFormat.supportedLocalesOf(['fr', 'pl', 'hi', 'ar', 'en', 'en-US']).join(',') !== 'fr,pl,en-US') {
  throw 'projected public RelativeTime locale inventory';
}
if (new Intl.RelativeTimeFormat('hi').resolvedOptions().locale !== 'en-US' ||
    new Intl.RelativeTimeFormat('he').resolvedOptions().locale !== 'en-US' ||
    new Intl.ListFormat('fr').resolvedOptions().locale !== 'en-US') {
  throw 'independent component filters and required fallback';
}
const french = new Intl.RelativeTimeFormat('fr-FR', { numeric: 'auto' });
const polish = new Intl.RelativeTimeFormat('pl-PL', { numeric: 'auto' });
if (french.resolvedOptions().locale !== 'fr' || polish.resolvedOptions().locale !== 'pl' ||
    french.format(-1, 'day') !== 'hier' || polish.format(-1, 'day') !== 'wczoraj') {
  throw 'actual selected RelativeTime templates';
}
const literalParts = french.formatToParts(-1, 'day');
if (literalParts.length !== 1 || literalParts[0].type !== 'literal' || literalParts[0].value !== 'hier') {
  throw 'selected Auto literal through actual parts wire';
}
for (const style of ['long', 'short', 'narrow']) {
  const formatter = new Intl.RelativeTimeFormat('pl', { style, numeric: 'always' });
  for (const unit of ['second', 'minute', 'hour', 'day', 'week', 'month', 'quarter', 'year']) {
    const parts = formatter.formatToParts(2, unit);
    if (parts.map(part => part.value).join('') !== formatter.format(2, unit) || parts.length === 0) {
      throw 'selected complete RelativeTime fields';
    }
  }
}
const arabicDigits = new Intl.RelativeTimeFormat('fr-u-nu-arab', { numeric: 'always' });
if (arabicDigits.resolvedOptions().numberingSystem !== 'arab' ||
    !arabicDigits.format(2, 'day').includes('٢') ||
    new Intl.NumberFormat('hi').resolvedOptions().locale !== 'hi' ||
    new Intl.RelativeTimeFormat('ar').resolvedOptions().locale !== 'en-US') {
  throw 'complete Number foundation does not widen RelativeTime support';
}
print('intl-combined-projection:ok');
262;
