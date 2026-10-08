function check(value, label) { if (!value) throw label; }
var date = new Date(0);
check(date.getHours() === 19 && date.getDate() === 31 && date.getFullYear() === 1969, 'selected local projection');
check(date.getUTCHours() === 0 && date.getUTCDate() === 1 && date.getUTCFullYear() === 1970, 'UTC projection stays independent');
check(date.toISOString() === '1970-01-01T00:00:00.000Z', 'ISO string stays UTC');
check(date.toUTCString() === 'Thu, 01 Jan 1970 00:00:00 GMT', 'UTC string stays UTC');
check(date.toGMTString() === 'Thu, 01 Jan 1970 00:00:00 GMT', 'Annex B GMT string stays UTC');
check(Date.UTC(1970, 0, 1, 0, 0, 0, 0) === 0, 'Date.UTC ignores selected zone');
check(date.setUTCFullYear(2000, 0, 2) === 946771200000, 'UTC year/date setter');
check(date.setUTCMonth(1, 3) === 949536000000, 'UTC month setter');
check(date.setUTCDate(4) === 949622400000, 'UTC date setter');
check(date.setUTCHours(5, 6, 7, 8) === 949640767008, 'UTC hours setter');
check(date.setUTCMinutes(9, 10, 11) === 949640950011, 'UTC minute setter');
check(date.setUTCSeconds(12, 13) === 949640952013, 'UTC seconds setter');
check(date.setUTCMilliseconds(14) === 949640952014, 'UTC millisecond setter');
print('ok');
262;
