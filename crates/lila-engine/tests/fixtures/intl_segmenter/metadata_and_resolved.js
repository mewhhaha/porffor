function check(value, message) { if (!value) throw new Error(message); }
function descriptor(object, key, writable, enumerable, configurable) {
  var d = Object.getOwnPropertyDescriptor(object, key);
  check(d.writable === writable && d.enumerable === enumerable && d.configurable === configurable, String(key));
}
check(Intl.Segmenter.name === 'Segmenter' && Intl.Segmenter.length === 0, 'constructor metadata');
descriptor(Intl, 'Segmenter', true, false, true);
descriptor(Intl.Segmenter, 'prototype', false, false, false);
check(Intl.Segmenter.supportedLocalesOf.length === 1, 'supported length');
check(Intl.Segmenter.prototype.segment.length === 1, 'segment length');
check(Intl.Segmenter.prototype.resolvedOptions.length === 0, 'resolved length');
descriptor(Intl.Segmenter.prototype, 'segment', true, false, true);
descriptor(Intl.Segmenter.prototype, 'resolvedOptions', true, false, true);
var tag = Object.getOwnPropertyDescriptor(Intl.Segmenter.prototype, Symbol.toStringTag);
check(tag.value === 'Intl.Segmenter' && !tag.writable && !tag.enumerable && tag.configurable, 'Segmenter tag');
var s = new Intl.Segmenter('sr', { granularity: 'word' });
var first = s.resolvedOptions(), second = s.resolvedOptions();
check(Object.keys(first).join(',') === 'locale,granularity', 'resolved order');
check(first.locale === 'sr' && first.granularity === 'word', 'actual admitted Serbian');
check(Object.getPrototypeOf(first) === Object.prototype && first !== second, 'fresh resolved');
descriptor(first, 'locale', true, true, true); descriptor(first, 'granularity', true, true, true);
first.locale = 'bad'; check(s.resolvedOptions().locale === 'sr', 'private locale');
check(Object.prototype.toString.call(s) === '[object Intl.Segmenter]', 'instance tag');
print('ok metadata_and_resolved'); 262;
