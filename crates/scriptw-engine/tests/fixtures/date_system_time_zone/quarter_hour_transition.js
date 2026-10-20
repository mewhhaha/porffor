function check(value, label) { if (!value) throw label; }
var date = new Date(1986, 0, 1, 0, 5);
check(date.getTime() === 504902100000, 'quarter-hour gap uses pre-jump offset');
check(date.getHours() === 0 && date.getMinutes() === 20 && date.getTimezoneOffset() === -345, 'quarter-hour displacement');
check(Date.parse('1986-01-01T00:05:00') === 504902100000, 'quarter-hour missing-zone parse');
var before = new Date(504901740000);
check(before.getFullYear() === 1985 && before.getHours() === 23 && before.getMinutes() === 59, 'pre-gap independent witness');
check(before.setFullYear(1986, 0, 1) === 504987240000, 'local full year uses actual selected offset');
check(new Date(504901800000).getMinutes() === 15, 'transition endpoint projects first post-gap minute');
print('ok');
262;
