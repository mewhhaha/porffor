function check(value, label) { if (!value) throw label; }
check(new Date(2021, 2, 14, 2, 30).getTime() === 1615707000000, 'gap uses offset before jump');
check(Date.parse('2021-03-14T02:30:00') === 1615707000000, 'missing-zone parse shares compatible gap');
var gap = new Date(1615703400000);
check(gap.setHours(2, 30) === 1615707000000 && gap.getHours() === 3, 'local setter crosses gap');
check(new Date(2021, 10, 7, 1, 30).getTime() === 1636263000000, 'fold chooses earliest epoch');
check(Date.parse('2021-11-07T01:30:00') === 1636263000000, 'missing-zone fold parse');
var fold = new Date(1636259400000);
check(fold.setHours(1, 30) === 1636263000000 && fold.getTimezoneOffset() === 240, 'local setter chooses early fold offset');
check(Date.parse('2021-03-14') === 1615680000000, 'date-only parse stays UTC');
check(Date.parse('2021-03-14T02:30:00Z') === 1615689000000, 'explicit Z stays UTC');
check(Date.parse('2021-11-07T01:30:00-05:00') === 1636266600000, 'explicit offset selects later fold');
check(new Date(1636266600000).getTimezoneOffset() === 300, 'offset uses actual formatted epoch');
print('ok');
262;
