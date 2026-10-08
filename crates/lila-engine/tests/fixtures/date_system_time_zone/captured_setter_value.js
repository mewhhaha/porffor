function check(value, label) { if (!value) throw label; }
var date = new Date(0), trace = [];
function argument(name, value) { return {valueOf() { trace.push(name); date.setTime(946684800000); return value; }}; }
check(date.setHours(argument('hour', 2), argument('minute', 30), argument('second', 5), argument('millisecond', 6)) === 5405006, 'setHours uses captured local date');
check(trace.join(',') === 'hour,minute,second,millisecond' && date.getTime() === 5405006, 'setter order and final store');
date.setTime(2592000000);
trace = [];
check(date.setMonth(argument('month', 1), argument('date', 15)) === 3888000000, 'setMonth uses captured year and clock');
check(trace.join(',') === 'month,date', 'setMonth supplied date order');
date.setTime(86400000);
check(date.setDate(argument('date', 3)) === 172800000, 'setDate uses captured month');
date.setTime(0);
check(date.setMilliseconds(argument('millisecond', 7)) === 7, 'setMilliseconds uses captured clock');
var marker = {}, caught, later = 0;
date.setTime(0);
try { date.setHours({valueOf() { date.setTime(123); throw marker; }}, {valueOf() { later++; return 0; }}); }
catch (error) { caught = error; }
check(caught === marker && later === 0 && date.getTime() === 123, 'abrupt conversion preserves mutation and identity');
print('ok');
262;
