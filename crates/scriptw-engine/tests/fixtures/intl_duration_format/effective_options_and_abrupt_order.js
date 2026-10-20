function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, fn, message) { var caught; try { fn(); } catch (error) { caught = error; } check(caught instanceof kind, message); }
var units = ['years','months','weeks','days','hours','minutes','seconds','milliseconds','microseconds','nanoseconds'];
var keys = ['localeMatcher','numberingSystem','style'];
for (var unit of units) { keys.push(unit); keys.push(unit + 'Display'); } keys.push('fractionalDigits');
var seen = [], options = new Proxy({}, { get(t, key) { seen.push(key); return undefined; } });
new Intl.DurationFormat('en', options);
check(seen.join(',') === keys.join(','), 'complete option read order');
for (var style of ['long','short','narrow']) {
  var result = new Intl.DurationFormat('en', { style: style }).resolvedOptions();
  check(result.style === style, 'zero-based style preserved');
  for (var name of units) check(result[name] === style && result[name + 'Display'] === 'auto', 'text style defaults');
}
var numeric = new Intl.DurationFormat('en', { hours: 'numeric' }).resolvedOptions();
check(numeric.hours === 'numeric' && numeric.hoursDisplay === 'always', 'explicit numeric hour');
check(numeric.minutes === '2-digit' && numeric.seconds === '2-digit', 'following width promotion');
check(numeric.milliseconds === 'numeric' && numeric.millisecondsDisplay === 'auto', 'fractional defaults');
seen = [];
throws(RangeError, function () { new Intl.DurationFormat('en', new Proxy({hours:'numeric', minutes:'long'}, {
  get(t, key) { seen.push(key); return t[key]; }
})); }, 'invalid style sequence');
check(seen[seen.length - 1] === 'minutesDisplay' && seen.indexOf('seconds') === -1, 'validate after this display before next style');
seen = [];
throws(RangeError, function () { new Intl.DurationFormat('en', new Proxy({style:'digital', millisecondsDisplay:'always'}, {
  get(t, key) { seen.push(key); return t[key]; }
})); }, 'fractional always invalid');
check(seen[seen.length - 1] === 'millisecondsDisplay', 'fractional validation follows display Get');
var coercions = [], styleValue = { toString() { coercions.push('style-string'); return 'narrow'; } };
new Intl.DurationFormat('en', { get style() { coercions.push('style'); return styleValue; }, get years() { coercions.push('years'); } });
check(coercions.join(',') === 'style,style-string,years', 'option ToString precedes next Get');
check(new Intl.DurationFormat('en', { fractionalDigits: 3.9 }).resolvedOptions().fractionalDigits === 3, 'GetNumberOption floors');
print('ok effective_options_and_abrupt_order'); 262;
