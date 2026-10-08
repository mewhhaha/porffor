function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, fn, message) { var caught; try { fn(); } catch (error) { caught = error; } check(caught instanceof kind, message); }
var order = ['days','hours','microseconds','milliseconds','minutes','months','nanoseconds','seconds','weeks','years'];
var formatter = new Intl.DurationFormat('en'), seen = [];
var invalid = new Proxy({days:1,hours:-1}, { get(t, key) { seen.push(key); return t[key]; } });
throws(RangeError, function () { formatter.format(invalid); }, 'uniform sign');
check(seen.join(',') === order.join(','), 'aggregate sign follows all ten reads');
for (var field of ['years','months','weeks']) {
  var bad = {}; bad[field] = 4294967296;
  throws(RangeError, function () { formatter.format(bad); }, 'calendar field exact bound');
}
throws(RangeError, function () { formatter.format({seconds:9007199254740992}); }, 'normalized seconds limit');
throws(RangeError, function () { formatter.format({seconds:9007199254740991,milliseconds:1000}); }, 'exact sum reaches limit');
check(typeof formatter.format({seconds:9007199254740991,milliseconds:999,microseconds:999,nanoseconds:999}) === 'string', 'exact sum below limit');
check(typeof formatter.format({nanoseconds:1180591620717411303424}) === 'string', 'integral Number wider than i64 remains admissible');
print('ok completed_aggregate_sign_and_bounds'); 262;
