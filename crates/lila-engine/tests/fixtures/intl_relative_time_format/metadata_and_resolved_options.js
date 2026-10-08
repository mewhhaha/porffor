function check(v, label) { if (!v) throw new Error(label); }
function same(a, b, label) { if (!Object.is(a, b)) throw new Error(label); }
same(Intl.RelativeTimeFormat.length, 0, 'constructor length');
same(Intl.RelativeTimeFormat.name, 'RelativeTimeFormat', 'constructor name');
same(Intl.RelativeTimeFormat.supportedLocalesOf.length, 1, 'supportedLocalesOf length');
same(Intl.RelativeTimeFormat.prototype.format.length, 2, 'format length');
same(Intl.RelativeTimeFormat.prototype.formatToParts.length, 2, 'parts length');
same(Intl.RelativeTimeFormat.prototype.resolvedOptions.length, 0, 'resolved length');
same(Object.prototype.toString.call(new Intl.RelativeTimeFormat('en-US')), '[object Intl.RelativeTimeFormat]', 'toStringTag');
var d = Object.getOwnPropertyDescriptor(Intl.RelativeTimeFormat.prototype, 'format');
check(d.writable && !d.enumerable && d.configurable && typeof d.value === 'function', 'format data method');
var rtf = new Intl.RelativeTimeFormat('en-US', {style:'short', numeric:'auto'});
var a = rtf.resolvedOptions(), b = rtf.resolvedOptions();
same(Object.keys(a).join(','), 'locale,style,numeric,numberingSystem', 'resolved property order');
same(a.locale, 'en-US', 'locale'); same(a.style, 'short', 'style'); same(a.numeric, 'auto', 'numeric'); same(a.numberingSystem, 'latn', 'numbering');
check(a !== b, 'fresh resolved object');
for (var key of Object.keys(a)) { var p = Object.getOwnPropertyDescriptor(a, key); check(p.writable && p.enumerable && p.configurable, 'resolved descriptor'); }
a.numeric = 'always'; same(rtf.format(0, 'day'), 'today', 'resolved object cannot mutate brand');
print('ok metadata_and_resolved_options');
262;
