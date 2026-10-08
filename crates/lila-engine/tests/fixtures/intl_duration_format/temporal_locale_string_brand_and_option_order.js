function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, fn, message) { var caught; try { fn(); } catch (error) { caught = error; } check(caught instanceof kind, message); }
var method = Temporal.Duration.prototype.toLocaleString;
var poison = new Proxy({}, { get() { throw new Error('brand must precede argument observation'); } });
for (var receiver of [undefined,null,1,'duration',{},Temporal.Duration.prototype,new Proxy(new Temporal.Duration(),{})]) {
  throws(TypeError,function () { method.call(receiver,poison,poison); },'Temporal brand before locales/options');
}
var duration = new Temporal.Duration(0,0,0,0,1), seen = [];
var locales = { get length() { seen.push('length'); return 1; }, get 0() { seen.push('locale'); return 'en'; } };
var options = new Proxy({}, {get(t,key) { seen.push(key); return undefined; }});
duration.toLocaleString(locales,options);
var expected = ['length','locale','localeMatcher','numberingSystem','style'];
for (var field of ['years','months','weeks','days','hours','minutes','seconds','milliseconds','microseconds','nanoseconds']) { expected.push(field); expected.push(field+'Display'); } expected.push('fractionalDigits');
check(seen.join(',') === expected.join(','),'Temporal method preserves the actual constructor options sequence');
throws(TypeError,function () { duration.toLocaleString('en',null); },'Temporal locale string strict options');
check(method.length === 0 && method.name === 'toLocaleString','preserved method metadata');
check(duration.toString() === 'PT1H' && duration.toJSON() === 'PT1H','ISO sibling methods preserve their semantics');
print('ok temporal_locale_string_brand_and_option_order'); 262;
