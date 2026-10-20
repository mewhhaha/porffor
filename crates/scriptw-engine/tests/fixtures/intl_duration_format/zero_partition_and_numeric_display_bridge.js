function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, fn, message) { var caught; try { fn(); } catch (error) { caught = error; } check(caught instanceof kind, message); }
var long = new Intl.DurationFormat('en',{style:'long'});
check(long.format({years:0}) === '' && long.formatToParts({years:0}).length === 0, 'honest zero-part extent');
var numeric = new Intl.DurationFormat('en',{style:'digital',hoursDisplay:'auto',minutesDisplay:'auto'});
check(numeric.format({seconds:3}) === '03', 'auto zero leading fields removed');
check(numeric.format({hours:1,seconds:3}) === '1:00:03', 'zero minute bridge retained between numeric endpoints');
check(numeric.formatToParts({hours:1,seconds:3}).map(p => p.value).join('') === '1:00:03', 'bridge exact parts');
print('ok zero_partition_and_numeric_display_bridge'); 262;
