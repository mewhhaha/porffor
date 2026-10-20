function check(value, label) { if (!value) throw label; }
var max = 8640000000000000, min = -8640000000000000;
check(new Date(275760, 8, 13, 1).getTime() === max, 'raw local upper coordinate maps inside TimeClip');
check(Date.parse('+275760-09-13T01:00:00.000') === max, 'upper missing-zone parse selects before TimeClip');
check(Number.isNaN(new Date(275760, 8, 13, 1, 0, 0, 1).getTime()), 'selected upper neighbor is invalid NaN');
check(Number.isNaN(Date.parse('+275760-09-13T01:00:00.001')), 'selected parse upper neighbor is invalid NaN');
var upper = new Date(max);
check(upper.getHours() === 1 && upper.setHours(1, 0, 0, 0) === max, 'upper projection and local setter');
check(Number.isNaN(upper.setMilliseconds(1)) && Number.isNaN(upper.getTime()), 'upper setter TimeClip stores NaN');
check(new Date(-271821, 3, 20, 1).getTime() === min, 'lower endpoint inverse');
check(Date.parse('-271821-04-20T01:00:00.000') === min, 'lower endpoint parse');
check(Number.isNaN(new Date(-271821, 3, 20, 0, 59, 59, 999).getTime()), 'lower neighbor NaN without Temporal error');
check(Date.parse('+275760-09-13T00:00:00.000Z') === max, 'explicit UTC limit independent of configuration');
print('ok');
262;
