function checkNumberingProjection(value, message) { if (!value) throw new Error(message); }
var globalNumbering = Intl.supportedValuesOf('numberingSystem');
checkNumberingProjection(globalNumbering.length === 78 && globalNumbering.includes('deva') && globalNumbering.includes('beng'), 'full admitted global digit authority');
for (var locale of ['en-US', 'fr', 'ar-EG']) {
  var number = new Intl.NumberFormat(locale, {numberingSystem: 'deva', useGrouping: false, maximumFractionDigits: 0});
  checkNumberingProjection(number.resolvedOptions().numberingSystem === 'deva' && number.format(12) === '१२', 'selected genuine Number digit association');
  var numberParts = number.formatToParts(12);
  checkNumberingProjection(numberParts.map(part => part.value).join('') === number.format(12), 'actual selected Number parts');
  var date = new Intl.DateTimeFormat(locale, {numberingSystem: 'deva', calendar: 'gregory', timeZone: 'UTC', year: 'numeric'});
  checkNumberingProjection(date.resolvedOptions().numberingSystem === 'deva', 'selected genuine DateTime digit association');
  var dateParts = date.formatToParts(946684800000);
  checkNumberingProjection(dateParts.some(part => part.type === 'year' && part.value === '२०००'), 'original full digit kernel renders selected year');
  checkNumberingProjection(dateParts.map(part => part.value).join('') === date.format(946684800000), 'actual selected DateTime parts');
  var expectedDefault = locale === 'ar-EG' ? 'arab' : 'latn';
  checkNumberingProjection(new Intl.NumberFormat(locale).resolvedOptions().numberingSystem === expectedDefault, 'original per-locale Number default');
  checkNumberingProjection(new Intl.DateTimeFormat(locale).resolvedOptions().numberingSystem === expectedDefault, 'original per-locale DateTime default');
  checkNumberingProjection(new Intl.NumberFormat(locale, {numberingSystem: 'beng'}).resolvedOptions().numberingSystem === expectedDefault, 'omitted Number option uses original default');
  checkNumberingProjection(new Intl.DateTimeFormat(locale, {numberingSystem: 'beng'}).resolvedOptions().numberingSystem === expectedDefault, 'omitted DateTime option uses original default');
  for (var Constructor of [Intl.NumberFormat, Intl.DateTimeFormat]) {
    var omitted = new Constructor(locale + '-u-nu-beng');
    checkNumberingProjection(omitted.resolvedOptions().numberingSystem === expectedDefault && !omitted.resolvedOptions().locale.includes('-u-nu-beng'), 'omitted extension is removed');
    var supported = new Constructor(locale + '-u-nu-deva', {numberingSystem: 'beng'});
    checkNumberingProjection(supported.resolvedOptions().numberingSystem === 'deva', 'unsupported option preserves supported extension');
  }
}
checkNumberingProjection(new Intl.RelativeTimeFormat('en-US', {numberingSystem: 'deva', numeric: 'always'}).format(2, 'day').includes('२'), 'dependent RelativeTime consumes same selected Number owner');
checkNumberingProjection(new Intl.DurationFormat('en-US', {numberingSystem: 'deva', style: 'digital'}).format({hours: 2, minutes: 3}).includes('२'), 'dependent Duration consumes same selected Number owner');
checkNumberingProjection(new Intl.RelativeTimeFormat('en-US', {numberingSystem: 'beng'}).resolvedOptions().numberingSystem === 'latn', 'private RelativeTime domain obeys projection');
checkNumberingProjection(new Intl.DurationFormat('en-US', {numberingSystem: 'beng'}).resolvedOptions().numberingSystem === 'latn', 'private Duration domain obeys projection');
checkNumberingProjection(new Intl.Locale('ar-EG').getNumberingSystems().includes('arab'), 'independent original Locale numbering preferences');
checkNumberingProjection(Intl.supportedValuesOf('calendar').length === 16, 'full unrelated calendar kernels');
checkNumberingProjection(new Intl.DateTimeFormat('en-US', {timeZone: 'America/New_York'}).resolvedOptions().timeZone === 'America/New_York', 'full named-zone authority');
print('intl-numbering-projection:ok');
262;
