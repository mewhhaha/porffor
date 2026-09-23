function check(value, message) { if (!value) throw new Error(message); }
const normal = new Intl.NumberFormat('en-US').resolvedOptions();
check(Object.keys(normal).join(',') === 'locale,numberingSystem,style,minimumIntegerDigits,minimumFractionDigits,maximumFractionDigits,useGrouping,notation,signDisplay,roundingIncrement,roundingMode,roundingPriority,trailingZeroDisplay', 'default property order');
const currency = new Intl.NumberFormat('en-US', {style:'currency',currency:'kwd',currencyDisplay:'code',currencySign:'accounting',notation:'compact',roundingPriority:'morePrecision'}).resolvedOptions();
check(Object.keys(currency).join(',') === 'locale,numberingSystem,style,currency,currencyDisplay,currencySign,minimumIntegerDigits,minimumFractionDigits,maximumFractionDigits,minimumSignificantDigits,maximumSignificantDigits,useGrouping,notation,compactDisplay,signDisplay,roundingIncrement,roundingMode,roundingPriority,trailingZeroDisplay', 'currency table order');
check(currency.currency === 'KWD' && currency.maximumFractionDigits === 3, 'normalized currency and nonstandard notation defaults');
const unit = new Intl.NumberFormat('en-US', {style:'unit',unit:'meter-per-second',unitDisplay:'long',minimumSignificantDigits:2,maximumSignificantDigits:4}).resolvedOptions();
check(!('currency' in unit) && !('currencyDisplay' in unit) && !('currencySign' in unit) && !('minimumFractionDigits' in unit) && !('compactDisplay' in unit), 'inactive options omitted');
check(unit.unit === 'meter-per-second' && unit.roundingPriority === 'auto', 'unit and computed priority');
const compact = new Intl.NumberFormat('en-US', {notation:'compact'}).resolvedOptions();
check(compact.minimumFractionDigits === 0 && compact.maximumFractionDigits === 0 && compact.minimumSignificantDigits === 1 && compact.maximumSignificantDigits === 2 && compact.roundingPriority === 'morePrecision', 'compact no-digit defaults');
for (const row of [['JPY',0],['KWD',3],['USD',2],['ZZZ',2]]) {
  const result = new Intl.NumberFormat('en-US', {style:'currency',currency:row[0]}).resolvedOptions();
  check(result.minimumFractionDigits === row[1] && result.maximumFractionDigits === row[1], 'shared currency digits ' + row[0]);
}
for (const word of ['true','false']) {
  check(new Intl.NumberFormat('en-US', {useGrouping:word}).resolvedOptions().useGrouping === 'auto', 'string grouping default');
  check(new Intl.NumberFormat('en-US', {useGrouping:word,notation:'compact'}).resolvedOptions().useGrouping === 'min2', 'compact grouping default');
}
for (const object of [normal,currency,unit,compact]) for (const key of Object.keys(object)) {
  const descriptor = Object.getOwnPropertyDescriptor(object,key);
  check(descriptor.writable && descriptor.enumerable && descriptor.configurable, 'resolved data descriptors');
}
print('ok resolved option order');
