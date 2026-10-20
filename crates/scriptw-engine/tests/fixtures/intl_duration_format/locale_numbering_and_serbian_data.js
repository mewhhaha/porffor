function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, fn, message) { var caught; try { fn(); } catch (error) { caught = error; } check(caught instanceof kind, message); }
var locales = Intl.DurationFormat.supportedLocalesOf(['sr-Thai-RS','en-US','sr','zh-CN','xx'],{localeMatcher:'lookup'});
check(locales.join(',') === 'sr-Thai-RS,en-US,sr,zh-CN', 'service finite supported requested tags');
check(new Intl.DurationFormat('sr-Thai-RS',{localeMatcher:'lookup'}).resolvedOptions().locale === 'sr', 'parent locale resolution');
var sr = new Intl.DurationFormat('sr',{style:'digital'}), bag = {hours:1,minutes:2,seconds:3};
check(sr.format(bag) === '1.02.03', 'genuine Serbian digital pattern');
check(sr.formatToParts(bag).filter(p => !Object.prototype.hasOwnProperty.call(p,'unit')).map(p => p.value).join('') === '..', 'captured separators retain text');
for (var numbering of ['latn','arab','tols']) {
  var formatter = new Intl.DurationFormat('en-u-nu-' + numbering,{style:'digital'});
  check(formatter.resolvedOptions().numberingSystem === numbering, 'shared admitted numbering');
  check(formatter.formatToParts(bag).map(p => p.value).join('') === formatter.format(bag), 'actual numbering partition');
}
check(new Intl.DurationFormat('en-u-nu-arab',{numberingSystem:'latn'}).resolvedOptions().numberingSystem === 'latn', 'explicit numbering overrides extension');
throws(RangeError,function () { new Intl.DurationFormat('en',{numberingSystem:'bad_name'}); },'numbering grammar');
check(new Intl.DurationFormat('en',{numberingSystem:'unknown'}).resolvedOptions().numberingSystem === 'latn', 'valid unavailable numbering resolves genuine default');
print('ok locale_numbering_and_serbian_data'); 262;
