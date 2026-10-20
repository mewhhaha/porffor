function check(value, label) { if (!value) throw label; }
var date = new Date(NaN), trace = [];
function argument(name, value) { return {valueOf() { trace.push(name); date.setTime(123); return value; }}; }
check(Number.isNaN(date.setMonth(argument('month', 1), argument('date', 2))), 'captured invalid month result');
check(date.getTime() === 123 && trace.join(',') === 'month,date', 'invalid month coerces optional date but does not store');
date.setTime(NaN); trace = [];
check(Number.isNaN(date.setHours(argument('hour', 1), argument('minute', 2), argument('second', 3), argument('millisecond', 4))), 'captured invalid hours result');
check(date.getTime() === 123 && trace.join(',') === 'hour,minute,second,millisecond', 'invalid hours performs all supplied conversions without store');
date.setTime(NaN);
check(date.setFullYear(argument('year', 2000)) === 946681200000, 'setFullYear recovers with local zero base');
date.setTime(NaN);
check(date.setYear(argument('year', 99.9)) === 915145200000, 'setYear recovers and truncates full year');
date.setTime(NaN);
check(date.setUTCFullYear(argument('year', 2000)) === 946684800000, 'UTC full year recovery ignores configured zone');
date.setTime(0);
check(Number.isNaN(date.setFullYear(NaN)) && Number.isNaN(date.getTime()), 'full year NaN stores invalid Date');
print('ok');
262;
