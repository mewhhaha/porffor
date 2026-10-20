function check(value, label) { if (!value) throw label; }
var max = 8640000000000000, min = -8640000000000000;
check(new Date(-271821, 3, 19, 23).getTime() === min, 'raw local lower coordinate maps inside TimeClip');
check(Date.parse('-271821-04-19T23:00:00.000') === min, 'lower missing-zone parse selects before TimeClip');
check(Number.isNaN(new Date(-271821, 3, 19, 22, 59, 59, 999).getTime()), 'selected lower neighbor is invalid NaN');
check(Number.isNaN(Date.parse('-271821-04-19T22:59:59.999')), 'selected parse lower neighbor is invalid NaN');
var lower = new Date(min);
check(lower.getDate() === 19 && lower.getHours() === 23 && lower.setHours(23, 0, 0, 0) === min, 'lower contextual projection and setter');
check(Number.isNaN(lower.setMilliseconds(-1)) && Number.isNaN(lower.getTime()), 'lower setter TimeClip stores NaN');
check(new Date(275760, 8, 12, 23).getTime() === max, 'upper endpoint inverse');
check(Date.parse('+275760-09-12T23:00:00.000') === max, 'upper endpoint parse');
check(Number.isNaN(new Date(275760, 8, 12, 23, 0, 0, 1).getTime()), 'upper neighbor NaN without Temporal error');
print('ok');
262;
