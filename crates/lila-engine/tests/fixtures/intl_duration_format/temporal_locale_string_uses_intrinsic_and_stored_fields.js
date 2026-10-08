function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, fn, message) { var caught; try { fn(); } catch (error) { caught = error; } check(caught instanceof kind, message); }
var duration = new Temporal.Duration(1,2,0,3,4,5,6,7,8,9);
var bag = {years:1,months:2,days:3,hours:4,minutes:5,seconds:6,milliseconds:7,microseconds:8,nanoseconds:9};
for (var locale of ['en','fr','ar','sr']) for (var style of ['long','short','narrow','digital']) {
  var options = {style:style,fractionalDigits:3};
  check(duration.toLocaleString(locale,options) === new Intl.DurationFormat(locale,options).format(bag), 'real intrinsic localized partition');
}
var expected = new Intl.DurationFormat('sr',{style:'digital'}).format(bag), original = Intl.DurationFormat;
for (var field of ['years','months','weeks','days','hours','minutes','seconds','milliseconds','microseconds','nanoseconds']) {
  Object.defineProperty(duration,field,{get() { throw new Error('Temporal own getter'); }});
}
Intl.DurationFormat = function () { throw new Error('overwritten public constructor'); };
try { check(duration.toLocaleString('sr',{style:'digital'}) === expected, 'intrinsic survives public overwrite and stored fields avoid getters'); }
finally { Intl.DurationFormat = original; }
var foreign = __lilaCreateRealm().global, foreignDuration = new foreign.Temporal.Duration(0,0,0,0,1,2,3);
check(Temporal.Duration.prototype.toLocaleString.call(foreignDuration,'sr',{style:'digital'}) === new Intl.DurationFormat('sr',{style:'digital'}).format({hours:1,minutes:2,seconds:3}), 'local called function with foreign stored receiver');
check(foreign.Temporal.Duration.prototype.toLocaleString.call(new Temporal.Duration(0,0,0,0,1,2,3),'sr',{style:'digital'}) === new Intl.DurationFormat('sr',{style:'digital'}).format({hours:1,minutes:2,seconds:3}), 'foreign method real intrinsic service');
print('ok temporal_locale_string_uses_intrinsic_and_stored_fields'); 262;
