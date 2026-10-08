function check(value, label) { if (!value) throw label; }
var trace = [];
function argument(name, value) { return {valueOf() { trace.push(name); return value; }}; }
var target = Array.bind(null);
Object.defineProperty(target, 'prototype', {get() { trace.push('prototype'); return Date.prototype; }});
var date = Reflect.construct(Date, [argument('year', 1970), argument('month', 0), argument('day', 1), argument('hour', 1), argument('minute', 2), argument('second', 3), argument('millisecond', 4)], target);
check(trace.join(',') === 'year,month,day,hour,minute,second,millisecond,prototype', 'constructor converts all arguments before prototype');
check(date.getTime() === 123004 && date.getHours() === 1, 'configured constructor epoch');
check(new Date(-0.9, 0, 1).getTime() === -2208992400000, 'negative fraction becomes full year 1900');
check(new Date(99.9, 0, 1).getTime() === 915145200000, 'fractional 99 becomes full year 1999');
check(Date.UTC(-0.9, 0, 1) === -2208988800000, 'UTC full year truncation 1900');
check(Date.UTC(99.9, 0, 1) === 915148800000, 'UTC full year truncation 1999');
trace = [];
check(Number.isNaN(Date.UTC(argument('year', NaN), argument('month', 0), argument('day', 1), argument('hour', 0), argument('minute', 0), argument('second', 0), argument('millisecond', 0))), 'NaN UTC result');
check(trace.join(',') === 'year,month,day,hour,minute,second,millisecond', 'UTC coerces supplied arguments before NaN result');
var marker = {}, later = 0, caught;
try { new Date({valueOf() { throw marker; }}, {valueOf() { later++; return 0; }}); }
catch (error) { caught = error; }
check(caught === marker && later === 0, 'constructor abrupt identity and order');
var calls = 0;
var ignored = {valueOf() { calls++; throw 'Date call argument'; }};
check(typeof Date(ignored) === 'string' && calls === 0 && Date.now() === 1234, 'Date call ignores arguments and uses fixed clock');
print('ok');
262;
