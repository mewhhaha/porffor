function check(value, label) { if (!value) throw label; }
var gap = new Date(2021, 9, 3, 2, 15);
check(gap.getTime() === 1633189500000 && gap.getHours() === 2 && gap.getMinutes() === 45, 'half-hour gap preserves displacement');
check(Date.parse('2021-10-03T02:15:00') === 1633189500000, 'half-hour parse inverse');
var fold = new Date(2021, 3, 4, 1, 45);
check(fold.getTime() === 1617461100000 && fold.getTimezoneOffset() === -660, 'half-hour fold earliest epoch');
check(new Date(1617462900000).getMinutes() === 45 && new Date(1617462900000).getTimezoneOffset() === -630, 'later fold offset at exact epoch');
var setter = new Date(1633187700000);
check(setter.setHours(2, 15) === 1633189500000, 'half-hour local setter');
print('ok');
262;
