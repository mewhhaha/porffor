function checkDurationProjection(value, message) { if (!value) throw new Error(message); }
checkDurationProjection(
  Intl.DurationFormat.supportedLocalesOf(['sr-Thai-RS', 'en-US', 'fr-FR', 'hi', 'de']).join(',') === 'sr-Thai-RS,en-US,fr-FR',
  'Duration public support consumes only selected rows');
checkDurationProjection(new Intl.DurationFormat('fr-FR').resolvedOptions().locale === 'fr', 'selected Duration parent row');
checkDurationProjection(new Intl.DurationFormat('hi').resolvedOptions().locale === 'en-US', 'excluded Duration row uses actual default');
var durationBag = { years: 1, months: 2, weeks: 3, days: 4, hours: 5, minutes: 6, seconds: 7, milliseconds: 8, microseconds: 9, nanoseconds: 10 };
for (var durationLocale of ['en-US', 'fr', 'sr']) {
  for (var durationStyle of ['long', 'short', 'narrow', 'digital']) {
    var durationFormatter = new Intl.DurationFormat(durationLocale, { style: durationStyle, fractionalDigits: 3 });
    var durationParts = durationFormatter.formatToParts(durationBag);
    checkDurationProjection(durationParts.map(part => part.value).join('') === durationFormatter.format(durationBag), 'all selected template styles retain actual Number partitions');
    for (var durationPart of durationParts) {
      checkDurationProjection(Object.keys(durationPart).join(',') ===
        (Object.prototype.hasOwnProperty.call(durationPart, 'unit') ? 'type,value,unit' : 'type,value'), 'selected partition descriptors');
    }
    var temporalDuration = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
    checkDurationProjection(temporalDuration.toLocaleString(durationLocale, { style: durationStyle, fractionalDigits: 3 }) === durationFormatter.format(durationBag), 'Temporal consumes the selected Duration owner');
  }
}
for (var durationPair of [['fr', ' et '], ['sr', ' и ']]) {
  var durationUnits = [
    new Intl.NumberFormat(durationPair[0], { style: 'unit', unit: 'year', unitDisplay: 'long' }).format(1),
    new Intl.NumberFormat(durationPair[0], { style: 'unit', unit: 'day', unitDisplay: 'long' }).format(2)
  ];
  checkDurationProjection(new Intl.DurationFormat(durationPair[0], { style: 'long' }).format({ years: 1, days: 2 }) ===
    durationUnits.join(durationPair[1]), 'selected Duration retains private List rows independently of public List support');
}
checkDurationProjection(new Intl.DurationFormat('sr', { style: 'digital' }).format({ hours: 1, minutes: 2, seconds: 3 }) === '1.02.03', 'genuine selected Serbian separators');
checkDurationProjection(new Intl.DurationFormat('fr', { style: 'digital' }).format({ hours: 1, minutes: 2, seconds: 3 }) === '1:02:03', 'genuine selected French separators');
checkDurationProjection(new Intl.DurationFormat('hi', { style: 'digital' }).format({ hours: 1, minutes: 2, seconds: 3 }) === '1:02:03', 'excluded Duration uses the selected en-US data');
checkDurationProjection(new Intl.NumberFormat('hi').resolvedOptions().locale === 'hi' &&
  new Intl.PluralRules('hi').resolvedOptions().locale === 'hi' && Intl.getCanonicalLocales('iw-IL')[0] === 'he-IL', 'Duration projection retains complete Number and Locale foundations');
print('intl-duration-projection:ok');
262;
