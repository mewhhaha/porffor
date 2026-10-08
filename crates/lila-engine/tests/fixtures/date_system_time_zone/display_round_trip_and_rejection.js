function check(value, label) { if (!value) throw label; }
for (var epoch of [-2208988800000, -1000, 0, 1000, 1593561600000]) {
  var date = new Date(epoch);
  check(Date.parse(date.toString()) === epoch, 'selected local display round trip ' + epoch);
  check(new Date(date.toString()).getTime() === epoch, 'display constructor round trip ' + epoch);
  check(Date.parse(date.toUTCString()) === epoch, 'UTC display round trip ' + epoch);
  check(Date.parse(date.toISOString()) === epoch, 'ISO display round trip ' + epoch);
}
var prefix = 'Mon Jan 01 1900 00:09:21 GMT';
var malformed = [
  prefix + '+0010 (Europe/Paris, +00:09:21)',
  prefix + '-0009 (Europe/Paris, +00:09:21)',
  prefix + '+0009 (Europe/Paris, -00:09:21)',
  prefix + '+0009 (Europe/Paris, +00:09:60)',
  prefix + '+0009 (Europe/Paris, +00:60:21)',
  prefix + '+0009 (Europe/Paris, +24:09:21)',
  prefix + '+0009 (, +00:09:21)',
  prefix + '+0009 (Europe Paris, +00:09:21)',
  prefix + '+0009 (' + 'A'.repeat(256) + ', +00:09:21)',
  prefix + '+0009 (Europe/Paris, +00:09:21)junk'
];
for (var source of malformed) check(Number.isNaN(Date.parse(source)), 'malformed owned display ' + source);
check(Date.parse('Mon Jan 01 1900 00:09:21 GMT+0009 (Europe/Paris, +00:09:21)') === -2208988800000, 'valid exact seconds companion');
print('ok');
262;
