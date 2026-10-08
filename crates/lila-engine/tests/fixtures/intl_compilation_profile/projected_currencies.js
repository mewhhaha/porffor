function checkCurrencyProjection(value, message) { if (!value) throw new Error(message); }
checkCurrencyProjection(Intl.supportedValuesOf('currency').join(',') === 'EUR,JPY', 'actual selected supported currency catalogue');
for (var locale of ['en-US', 'fr']) {
  for (var currency of ['EUR', 'JPY']) {
    for (var display of ['symbol', 'narrowSymbol', 'code', 'name']) {
      var formatter = new Intl.NumberFormat(locale, {style: 'currency', currency: currency, currencyDisplay: display});
      var parts = formatter.formatToParts(12);
      checkCurrencyProjection(parts.map(part => part.value).join('') === formatter.format(12), 'selected genuine currency partition');
      checkCurrencyProjection(parts.some(part => part.type === 'currency'), 'selected symbol or name row');
      checkCurrencyProjection(formatter.resolvedOptions().currency === currency, 'actual original currency option');
    }
    var names = new Intl.DisplayNames(locale, {type: 'currency', fallback: 'none'});
    checkCurrencyProjection(typeof names.of(currency) === 'string' && names.of(currency) !== currency, 'actual selected DisplayNames currency data');
  }
  for (var display of ['symbol', 'narrowSymbol', 'code', 'name']) {
    var omitted = new Intl.NumberFormat(locale, {style: 'currency', currency: 'USD', currencyDisplay: display});
    checkCurrencyProjection(omitted.formatToParts(12).some(part => part.type === 'currency' && part.value === 'USD'), 'original omitted-code Number fallback');
  }
  checkCurrencyProjection(new Intl.DisplayNames(locale, {type: 'currency', fallback: 'none'}).of('USD') === undefined, 'actual omitted currency has no name');
  checkCurrencyProjection(new Intl.DisplayNames(locale, {type: 'currency', fallback: 'code'}).of('USD') === 'USD', 'original DisplayNames code fallback');
}
checkCurrencyProjection(new Intl.NumberFormat('en-US', {style: 'currency', currency: 'JPY'}).resolvedOptions().maximumFractionDigits === 0, 'selected currency fractions');
checkCurrencyProjection(new Intl.NumberFormat('en-US', {style: 'currency', currency: 'BHD'}).resolvedOptions().maximumFractionDigits === 3, 'global fractions remain complete for omitted codes');
checkCurrencyProjection(new Intl.DisplayNames('en-US', {type: 'region'}).of('US') === 'United States', 'other name domains remain complete');
checkCurrencyProjection(Intl.supportedValuesOf('numberingSystem').length === 78, 'complete global numbering authority');
checkCurrencyProjection(Intl.supportedValuesOf('calendar').length === 16, 'complete global calendar authority');
checkCurrencyProjection(new Intl.RelativeTimeFormat('fr').format(2.5, 'day') === 'dans 2,5 jours', 'same Number kernel for unrelated dependent formatting');
checkCurrencyProjection(new Intl.DurationFormat('sr', {style: 'digital', fractionalDigits: 3}).format({hours: 1, minutes: 2, seconds: 3, milliseconds: 500}) === '1.02.03,500', 'same private Duration Number kernel');
print('intl-currency-projection:ok');
262;
