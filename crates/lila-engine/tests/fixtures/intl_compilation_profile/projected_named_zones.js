function checkNamedZoneProjection(value, message) { if (!value) throw new Error(message); }
var knownZones = Intl.supportedValuesOf('timeZone');
checkNamedZoneProjection(knownZones.includes('America/New_York') && knownZones.includes('Europe/Paris') && knownZones.includes('UTC') && !knownZones.includes('US/Eastern'), 'complete genuine primary identity authority');
checkNamedZoneProjection(new Intl.Locale('fr-FR').getTimeZones().includes('Europe/Paris'), 'complete original country authority outside transition selection');
for (var locale of ['en-US', 'fr']) {
  var options = {timeZone: 'America/New_York', hour: 'numeric', minute: 'numeric', second: 'numeric', hourCycle: 'h23', timeZoneName: 'longOffset'};
  var zone = new Intl.DateTimeFormat(locale, options);
  var before = zone.formatToParts(1710053999000), after = zone.formatToParts(1710054000000);
  checkNamedZoneProjection(Number(before.find(part => part.type === 'hour').value) === 1 && Number(after.find(part => part.type === 'hour').value) === 3, 'original selected spring transition');
  checkNamedZoneProjection(before.map(part => part.value).join('') === zone.format(1710053999000) && after.map(part => part.value).join('') === zone.format(1710054000000), 'selected localized name and parts closure');
  var alias = new Intl.DateTimeFormat(locale, {timeZone: 'us/eastern', hour: 'numeric', minute: 'numeric', second: 'numeric', hourCycle: 'h23', timeZoneName: 'longOffset'});
  checkNamedZoneProjection(alias.format(1710053999000) === zone.format(1710053999000) && alias.format(1710054000000) === zone.format(1710054000000), 'actual selected-primary alias transition closure');
  var utc = new Intl.DateTimeFormat(locale, {timeZone: 'UTC', hour: 'numeric', hourCycle: 'h23'}).formatToParts(0);
  checkNamedZoneProjection(Number(utc.find(part => part.type === 'hour').value) === 0, 'mandatory UTC original data');
  var offset = new Intl.DateTimeFormat(locale, {timeZone: '+05:30', hour: 'numeric', minute: 'numeric', hourCycle: 'h23'}).formatToParts(0);
  checkNamedZoneProjection(Number(offset.find(part => part.type === 'hour').value) === 5 && Number(offset.find(part => part.type === 'minute').value) === 30, 'original independent fixed-offset algebra');
}
var instant = Temporal.Instant.fromEpochNanoseconds(1710054000000000000n);
checkNamedZoneProjection(instant.toZonedDateTimeISO('America/New_York').hour === 3 && instant.toZonedDateTimeISO('US/Eastern').hour === 3, 'Temporal uses retained original exact offset owner');
checkNamedZoneProjection(Temporal.ZonedDateTime.from({timeZone: 'America/New_York', year: 2024, month: 3, day: 10, hour: 2, minute: 30}).hour === 3, 'retained original gap topology');
checkNamedZoneProjection(Intl.supportedValuesOf('calendar').length === 16 && Intl.supportedValuesOf('numberingSystem').length === 78, 'complete unrelated global kernels');
print('intl-named-zone-projection:ok');
262;
