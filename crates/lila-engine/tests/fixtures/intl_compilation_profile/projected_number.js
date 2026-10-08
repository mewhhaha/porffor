function checkNumberProjection(value, message) { if (!value) throw new Error(message); }
var requestedNumberLocales = ['pl-PL', 'es-MX', 'en-US', 'fr', 'sr', 'ar', 'he'];
for (var numberService of [Intl.NumberFormat, Intl.PluralRules]) {
  var supportedNumberLocales = numberService.supportedLocalesOf(requestedNumberLocales);
  checkNumberProjection(supportedNumberLocales.join(',') === 'pl-PL,es-MX,en-US', 'coupled public Number and Plural inventory');
  var numberElement = Object.getOwnPropertyDescriptor(supportedNumberLocales, '0');
  checkNumberProjection(numberElement.writable && numberElement.enumerable && numberElement.configurable, 'fresh supported-locale descriptors');
  supportedNumberLocales[0] = 'changed';
  checkNumberProjection(numberService.supportedLocalesOf(requestedNumberLocales)[0] === 'pl-PL', 'supported-locale arrays are independent');
  checkNumberProjection(new numberService('es-MX').resolvedOptions().locale === 'es', 'selected real Spanish parent row');
  checkNumberProjection(new numberService('pl-PL').resolvedOptions().locale === 'pl', 'selected real Polish parent row');
  for (var excludedNumberLocale of ['fr', 'sr', 'ar', 'he']) {
    checkNumberProjection(new numberService(excludedNumberLocale).resolvedOptions().locale === 'en-US', 'private or omitted rows do not enter public resolution');
  }
}
for (var decimalLocale of ['es', 'pl']) {
  var decimalFormatter = new Intl.NumberFormat(decimalLocale, { useGrouping: false });
  checkNumberProjection(decimalFormatter.format(123.5) === '123,5', 'selected decimal symbols');
  checkNumberProjection(decimalFormatter.formatToParts(123.5).map(part => part.value).join('') === '123,5', 'selected decimal partitions');
  checkNumberProjection(decimalFormatter.format(123456789012345678901n) === '123456789012345678901', 'selected exact BigInt formatting');
  var rangeParts = decimalFormatter.formatRangeToParts(1, 2);
  checkNumberProjection(rangeParts.map(part => part.value).join('') === decimalFormatter.formatRange(1, 2) &&
    rangeParts.some(part => part.source === 'startRange') && rangeParts.some(part => part.source === 'endRange'), 'selected range policies and source attribution');
}
var polishPlural = new Intl.PluralRules('pl');
checkNumberProjection([1, 2, 5, 1.5].map(value => polishPlural.select(value)).join(',') === 'one,few,many,other', 'complete selected cardinal AST');
checkNumberProjection(polishPlural.selectRange(2, 3) === 'few', 'selected cardinal range matrix');
checkNumberProjection(new Intl.PluralRules('pl', { type: 'ordinal' }).select(1) === 'other', 'selected ordinal AST');
var polishCurrency = new Intl.NumberFormat('pl', { style: 'currency', currency: 'PLN', currencyDisplay: 'code' }).formatToParts(2);
checkNumberProjection(polishCurrency.some(part => part.type === 'currency' && part.value === 'PLN') &&
  polishCurrency.some(part => part.type === 'fraction' && part.value === '00'), 'retained currency metadata and partitions');
checkNumberProjection(new Intl.NumberFormat('es', { style: 'unit', unit: 'meter', unitDisplay: 'long' }).format(1) === '1 metro', 'selected Spanish unit pool');
checkNumberProjection(new Intl.NumberFormat('pl', { style: 'unit', unit: 'meter', unitDisplay: 'long' }).format(2) === '2 metry', 'selected Polish plural unit pool');
checkNumberProjection(new Intl.RelativeTimeFormat('fr').format(2.5, 'day') === 'dans 2,5 jours', 'RelativeTime retains its private French Number and Plural domain');
checkNumberProjection(new Intl.DurationFormat('sr', { style: 'digital', fractionalDigits: 3 }).format({ hours: 1, minutes: 2, seconds: 3, milliseconds: 500 }) === '1.02.03,500', 'Duration retains its private Serbian Number domain');
checkNumberProjection(new Temporal.Duration(0, 0, 0, 0, 1, 2, 3, 500).toLocaleString('sr', { style: 'digital', fractionalDigits: 3 }) === '1.02.03,500', 'Temporal intrinsic consumes selected private Duration dependencies');
checkNumberProjection(Intl.NumberFormat.supportedLocalesOf(['fr', 'sr']).length === 0 && Intl.PluralRules.supportedLocalesOf(['fr', 'sr']).length === 0, 'dependent formatters do not broaden public domains');
checkNumberProjection(new Intl.Locale('ar').numberingSystems[0] === 'latn' && new Intl.Locale('ar-EG').numberingSystems[0] === 'arab', 'complete default-numbering authority outside public formatting');
checkNumberProjection(Intl.supportedValuesOf('numberingSystem').length === 78 && Intl.supportedValuesOf('numberingSystem').includes('tols'), 'complete digit catalogue survives native projection');
checkNumberProjection(new Intl.NumberFormat('en-US-u-nu-tols', { useGrouping: false }).format(1234567890) === '𑷡𑷢𑷣𑷤𑷥𑷦𑷧𑷨𑷩𑷠', 'selected supplementary numbering digits');
checkNumberProjection(Intl.getCanonicalLocales('iw-IL')[0] === 'he-IL', 'complete Locale canonicalization foundation');
print('intl-number-projection:ok');
262;
