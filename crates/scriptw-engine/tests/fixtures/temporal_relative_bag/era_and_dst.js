function check(actual, expected, label) {
  if (actual !== expected) throw new Error(label + ': ' + actual + ' != ' + expected);
}
function hours(bag) {
  return Temporal.Duration.from({days: 1}).total({unit: 'hours', relativeTo: bag});
}
[
  {calendar: 'gregory', era: 'ce', eraYear: 2021},
  {calendar: 'buddhist', year: 2564},
  {calendar: 'roc', era: 'roc', eraYear: 110},
  {calendar: 'japanese', era: 'reiwa', eraYear: 3}
].forEach(function(fields) {
  fields.month = 3; fields.day = 13; fields.hour = 12;
  fields.timeZone = 'America/New_York';
  check(hours(fields), 23, fields.calendar + ' era/year conversion before DST');
});
check(hours({year: 2021, month: 11, day: 6, hour: 12, timeZone: 'America/New_York'}), 25, 'fall elapsed day');
check(hours({year: 2021, month: 10, day: 2, hour: 12, timeZone: 'Australia/Lord_Howe'}), 23.5, 'half-hour change');
check(hours({year: 2021, month: 2, day: 40, hour: 100, timeZone: undefined}), 24, 'plain bag constrains then retains only date');
function fold(offset) {
  var bag = {year: 2021, month: 11, day: 7, hour: 1, minute: 30, timeZone: 'America/New_York'};
  if (offset !== undefined) bag.offset = offset;
  return Temporal.Duration.compare({days: 1}, {hours: 24}, {relativeTo: bag});
}
check(fold(undefined), 1, 'missing offset chooses compatible earlier fold');
check(fold('-04:00'), 1, 'explicit earlier fold epoch');
check(fold('-05:00'), 0, 'explicit later fold epoch');
var zone = Temporal.ZonedDateTime.from('2000-01-01T00:00[America/New_York]');
Object.defineProperty(zone, 'timeZoneId', {get: function() {throw new Error('public zone getter');}});
check(hours({year: 2021, month: 3, day: 13, hour: 12, timeZone: zone}), 23, 'branded zone private slot');
check(Temporal.Duration.from({hours: 1}).total({unit: 'seconds', relativeTo: {year: 1900, month: 1, day: 1, timeZone: 'Europe/Paris', offset: '+00:09:21'}}), 3600, 'sub-minute offset exact acceptance');
262;
