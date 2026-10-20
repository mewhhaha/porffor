function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, fn, message) { var caught; try { fn(); } catch (error) { caught = error; } check(caught instanceof kind, message); }
var fields = ['years','months','weeks','days','hours','minutes','seconds','milliseconds','microseconds','nanoseconds'];
var duration = new Temporal.Duration(1,2,3,4,5,6,7,8,9,10);
var bag = {years:1,months:2,weeks:3,days:4,hours:5,minutes:6,seconds:7,milliseconds:8,microseconds:9,nanoseconds:10};
for (var field of fields) Object.defineProperty(duration, field, { get() { throw new Error('stored field getter'); } });
for (var style of ['long','short','narrow','digital']) {
  var formatter = new Intl.DurationFormat('en', {style:style});
  check(formatter.format(duration) === formatter.format(bag), 'stored brand fields bypass getters');
  check(formatter.format('P1Y2M3W4DT5H6M7.00800901S') === formatter.format(bag), 'real ISO string parser');
  check(formatter.format('PT0S') === formatter.format({years:0}), 'zero string');
  check(formatter.formatToParts(duration).map(p => p.value).join('') === formatter.format(bag), 'stored fields parts');
}
var saw = false;
var proxy = new Proxy(duration, { get(t,key) { saw = true; throw new Error('ordinary proxy field'); } });
throws(Error, function () { new Intl.DurationFormat('en').format(proxy); }, 'proxy takes ordinary bag path');
check(saw, 'proxy observation retained');
print('ok stored_temporal_duration_and_strings'); 262;
