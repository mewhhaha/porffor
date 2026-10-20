function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, fn, message) { var caught; try { fn(); } catch (error) { caught = error; } check(caught instanceof kind, message); }
var order = ['days','hours','microseconds','milliseconds','minutes','months','nanoseconds','seconds','weeks','years'];
var seen = [], bag = {};
for (var field of order) { (function (name) { Object.defineProperty(bag, name, { get() {
  seen.push('get ' + name); return { valueOf() { seen.push('number ' + name); return 1; } };
} }); })(field); }
var formatter = new Intl.DurationFormat('en'); formatter.format(bag);
var expected = []; for (var field of order) { expected.push('get ' + field); expected.push('number ' + field); }
check(seen.join(',') === expected.join(','), 'each Get immediately coerced before next Get');
seen = [];
var missing = new Proxy({}, { get(t, key) { seen.push(key); return undefined; } });
throws(TypeError, function () { formatter.format(missing); }, 'missing all fields');
check(seen.join(',') === order.join(','), 'missing record observes all ten fields');
check(typeof formatter.format({years:null}) === 'string', 'present null coerces to zero');
print('ok duration_field_sequence_and_coercions'); 262;
