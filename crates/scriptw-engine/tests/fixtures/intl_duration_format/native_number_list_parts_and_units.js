function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, fn, message) { var caught; try { fn(); } catch (error) { caught = error; } check(caught instanceof kind, message); }
var locales = ['ar','ar-EG','de','en','en-US','es','fr','hi','it','ja','ko','sr','zh','zh-Hans','zh-Hans-CN'];
var bag = {years:1,months:2,days:3,hours:4,minutes:5,seconds:6,milliseconds:7,microseconds:8,nanoseconds:9};
var allowed = ['literal','integer','group','decimal','fraction','minusSign','unit'];
for (var locale of locales) for (var style of ['long','short','narrow','digital']) {
  var formatter = new Intl.DurationFormat(locale, {style:style}), parts = formatter.formatToParts(bag);
  check(parts.map(p => p.value).join('') === formatter.format(bag), 'native text equals exact parts');
  for (var part of parts) {
    check(allowed.indexOf(part.type) !== -1 && typeof part.value === 'string', 'closed native part domain');
    if (!Object.prototype.hasOwnProperty.call(part,'unit')) check(part.type === 'literal', 'unitless separator is literal');
    else check(['year','month','week','day','hour','minute','second','millisecond','microsecond','nanosecond'].indexOf(part.unit) !== -1, 'real part unit');
    check(Object.keys(part).join(',') === (Object.prototype.hasOwnProperty.call(part,'unit') ? 'type,value,unit' : 'type,value'), 'own property insertion order');
  }
}
var formatter = new Intl.DurationFormat('en',{style:'long'});
check(formatter.format({hours:2}) === new Intl.NumberFormat('en',{style:'unit',unit:'hour',unitDisplay:'long'}).format(2), 'actual NumberFormat unit composition');
var unitStrings = [new Intl.NumberFormat('en',{style:'unit',unit:'year',unitDisplay:'long'}).format(1),
  new Intl.NumberFormat('en',{style:'unit',unit:'day',unitDisplay:'long'}).format(2)];
check(formatter.format({years:1,days:2}) === new Intl.ListFormat('en',{type:'unit',style:'long'}).format(unitStrings), 'actual ListFormat unit composition');
print('ok native_number_list_parts_and_units'); 262;
