function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var log = []; Object.defineProperty(Number.prototype, 'usage', {configurable: true, get() { log.push('boxed usage'); return 'search'; }});
var c = new Intl.Collator('de', 7); same(c.resolvedOptions().usage, 'search', 'constructor primitive boxed'); same(log.join('|'), 'boxed usage', 'boxed one Get'); delete Number.prototype.usage;
throws(TypeError, function() { new Intl.Collator('en-US', null); }, 'null options');
var locales = {length: 1, get 0() { log.push('locale'); return 'en-US'; }}; var options = {get localeMatcher() { log.push('matcher'); return 'lookup'; }};
log = []; var supported = Intl.Collator.supportedLocalesOf(locales, options); same(log.join('|'), 'locale|matcher', 'supported order'); same(supported[0], 'en-US', 'actual default locale admitted');
same(Object.getPrototypeOf(supported), Array.prototype, 'supported array prototype');
Object.defineProperty(Boolean.prototype, 'localeMatcher', {configurable: true, get() { log.push('boxed matcher'); return 'lookup'; }});
log = []; same(Intl.Collator.supportedLocalesOf('en-US', false)[0], 'en-US', 'static primitive boxed'); same(log.join('|'), 'boxed matcher', 'static one Get'); delete Boolean.prototype.localeMatcher;
same(Intl.Collator.supportedLocalesOf(['zz-ZZ'], {localeMatcher: 'lookup'}).length, 0, 'unavailable is not default fallback support');
print('ok option_boxing_and_supported');
262;
