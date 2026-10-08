function check(value, message) { if (!value) throw new Error(message); }
var requested = ['sr-Thai-RS', 'de', 'zh-CN', 'xyz', 'de'];
var result = Intl.Segmenter.supportedLocalesOf(requested, { localeMatcher: 'lookup' });
check(result.join(',') === 'sr-Thai-RS,de,zh-CN', 'preserve supported original requests');
check(result !== Intl.Segmenter.supportedLocalesOf(requested) && Object.getPrototypeOf(result) === Array.prototype, 'fresh actual array');
check(Intl.Segmenter.supportedLocalesOf(['sv', 'fi', 'el']).join(',') === 'sv,fi,el', 'genuine tailored locale domain');
check(new Intl.Segmenter(['xyz', 'ar']).resolvedOptions().locale === 'ar', 'first supported');
check(new Intl.Segmenter('EN').resolvedOptions().locale === 'en', 'canonical case');
check(new Intl.Segmenter('en-US-u-ca-hebrew').resolvedOptions().locale === 'en-US', 'no relevant extensions');
var primitives = [true, false, 'x', 3, Symbol('options'), 1n];
for (var i = 0; i < primitives.length; ++i) check(Intl.Segmenter.supportedLocalesOf(['en'], primitives[i]).join(',') === 'en', 'supported options ToObject');
var log = [];
var locales = { get length() { log.push('locales'); return 1; }, get 0() { log.push('locale0'); return 'sr'; } };
var options = { get localeMatcher() { log.push('matcher'); return 'lookup'; } };
Intl.Segmenter.supportedLocalesOf(locales, options);
check(log.join(',') === 'locales,locale0,matcher', 'supported observation order');
print('ok supported_locales_and_option_boxing'); 262;
