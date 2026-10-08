function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label); }
var log = [];
var locale = {toString() { log.push('locale'); return 'en-us'; }};
var options = {get localeMatcher() { log.push('matcher'); return {toString() { log.push('matcher.string'); return 'lookup'; }}; }};
var result = Intl.ListFormat.supportedLocalesOf.call(null, [locale, 'en-US', 'en-US-u-ca-buddhist'], options);
same(log.join(','), 'locale,matcher,matcher.string', 'canonicalize before options');
same(result.join(','), 'en-US,en-US-u-ca-buddhist', 'canonicalize/deduplicate and retain supported extension');
same(Object.getPrototypeOf(result), Array.prototype, 'supported array Realm');
var gets = 0, invalidLocaleError;
try { Intl.ListFormat.supportedLocalesOf(['not_a_tag'], new Proxy({}, {get() { gets++; return 'lookup'; }})); } catch (e) { invalidLocaleError = e; }
check(invalidLocaleError && invalidLocaleError.constructor === RangeError, 'invalid locale rejected');
same(gets, 0, 'locale failure before options');
var boxed = 0;
Object.defineProperty(Number.prototype, 'localeMatcher', {configurable: true, get() { boxed++; same(Object.getPrototypeOf(this), Number.prototype, 'primitive boxed in called Realm'); return 'lookup'; }});
same(Intl.ListFormat.supportedLocalesOf('en-US', 7)[0], 'en-US', 'static primitive options accepted');
same(boxed, 1, 'boxed matcher read once');
delete Number.prototype.localeMatcher;
var nullError;
try { Intl.ListFormat.supportedLocalesOf('en-US', null); } catch (e) { nullError = e; }
check(nullError && nullError.constructor === TypeError, 'null static options rejected');
same(Intl.ListFormat.supportedLocalesOf(['en-US', 'es', 'he'], {localeMatcher: 'lookup'}).join(','), 'en-US,es,he', 'actual pinned template locales admitted');
print('ok supported_locale_order_and_boxing');
262;
