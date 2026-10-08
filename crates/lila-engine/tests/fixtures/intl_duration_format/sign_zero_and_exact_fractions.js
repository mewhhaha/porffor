function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, fn, message) { var caught; try { fn(); } catch (error) { caught = error; } check(caught instanceof kind, message); }
var digital = new Intl.DurationFormat('en',{style:'digital'});
check(digital.format({seconds:-1}) === '-0:00:01', 'first displayed zero owns negative sign');
check(digital.formatToParts({seconds:-1}).filter(p => p.type === 'minusSign').length === 1, 'one minus sign');
var fields = ['years','months','weeks','days','hours','minutes','seconds','milliseconds','microseconds','nanoseconds'];
for (var field of fields) { var bag = {}; bag[field] = -0; check(digital.format(bag) === digital.format({seconds:0}), 'negative zero neutral'); }
var precise = new Intl.DurationFormat('en',{style:'digital',fractionalDigits:3});
check(precise.format({hours:1,minutes:2,seconds:3,milliseconds:123,microseconds:999,nanoseconds:999}) === '1:02:03.123', 'fraction truncation and exact composed fields');
check(precise.format({hours:-1,minutes:-2,seconds:-3}) === '-1:02:03.000', 'requested fractional zeroes');
check(digital.format({seconds:1,milliseconds:2,microseconds:3,nanoseconds:9007199254740991}) === '0:00:9007200.256743991', 'wide Number nanoseconds retain all nine places');
check(digital.format({milliseconds:4503599627370497000,microseconds:4503599627370495000000}) === '0:00:9007199254740991.975424', 'unsafe integer exact mathematical composition');
print('ok sign_zero_and_exact_fractions'); 262;
