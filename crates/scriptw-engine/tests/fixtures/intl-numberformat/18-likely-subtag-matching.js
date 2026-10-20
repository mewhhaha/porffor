function same(actual, expected, message) {
  if (!Object.is(actual, expected)) throw new Error(message + ': ' + actual);
}
var formatter = new Intl.NumberFormat('zh-TW');
same(formatter.format(NaN), '非數值', 'traditional NaN label');
same(NaN.toLocaleString('zh-TW'), '非數值', 'primitive delegates to the selected formatter');
same(new Intl.NumberFormat('zh-TW', {notation:'compact'}).format(987654321),
     '9.9億', 'traditional compact pattern');
same(new Intl.NumberFormat('zh-TW', {
  style:'unit', unit:'kilometer-per-hour', unitDisplay:'short'
}).format(-987), '-987 公里/小時', 'traditional unit label');
same(formatter.resolvedOptions().locale, 'zh-Hant-TW', 'best fit retains the matching physical locale');
same(new Intl.NumberFormat('zh-TW', {localeMatcher:'lookup'}).resolvedOptions().locale,
     'zh', 'lookup keeps prefix semantics');
same(new Intl.NumberFormat('zh-TW-u-nu-hanidec').format(123), '一二三', 'numbering extension survives matching');
same(Intl.NumberFormat.supportedLocalesOf(['zh-TW-u-nu-hanidec'])[0],
     'zh-TW-u-nu-hanidec', 'supported locales returns requested canonical spelling');
same(new Intl.NumberFormat('en-GB-fonipa').resolvedOptions().locale, 'en-GB',
     'specific regional fallback beats a less specific likely-subtag fallback');
print('ok likely subtag matching');
