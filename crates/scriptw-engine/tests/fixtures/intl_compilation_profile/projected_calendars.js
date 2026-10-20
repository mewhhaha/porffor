function checkCalendarProjection(value, message) { if (!value) throw new Error(message); }
var calendarKernels = Intl.supportedValuesOf('calendar');
checkCalendarProjection(calendarKernels.length === 16 && calendarKernels.includes('buddhist') && calendarKernels.includes('chinese'), 'full genuine global kernel authority');
for (var locale of ['en-US', 'fr']) {
  var selected = new Intl.DateTimeFormat(locale, {calendar: 'chinese', timeZone: 'UTC', year: 'numeric', month: 'long', day: 'numeric'});
  checkCalendarProjection(selected.resolvedOptions().calendar === 'chinese', 'selected genuine localized calendar');
  var parts = selected.formatToParts(946684800000);
  checkCalendarProjection(parts.map(part => part.value).join('') === selected.format(946684800000), 'selected Chinese field partition');
  checkCalendarProjection(parts.some(part => part.type === 'relatedYear') || parts.some(part => part.type === 'yearName'), 'real Chinese kernel and native fields');
  checkCalendarProjection(new Intl.DateTimeFormat(locale, {calendar: 'buddhist'}).resolvedOptions().calendar === 'gregory', 'omitted localized calendar uses original locale default');
  var omittedExtension = new Intl.DateTimeFormat(locale + '-u-ca-buddhist');
  checkCalendarProjection(omittedExtension.resolvedOptions().calendar === 'gregory' && !omittedExtension.resolvedOptions().locale.includes('-u-ca-buddhist'), 'unsupported extension is omitted');
  var overridden = new Intl.DateTimeFormat(locale + '-u-ca-chinese', {calendar: 'buddhist'});
  checkCalendarProjection(overridden.resolvedOptions().calendar === 'chinese', 'unsupported option preserves supported extension selection');
  checkCalendarProjection(new Intl.DateTimeFormat(locale).resolvedOptions().calendar === 'gregory', 'original mandatory default row');
}
checkCalendarProjection(new Intl.Locale('th-TH').getCalendars().includes('buddhist'), 'independent LocaleInfo calendar preference authority');
checkCalendarProjection(new Intl.DisplayNames('en-US', {type: 'calendar'}).of('buddhist') !== 'buddhist', 'unfiltered calendar name authority');
checkCalendarProjection(new Intl.DateTimeFormat('en-US', {calendar: 'chinese', timeZone: 'America/New_York'}).resolvedOptions().timeZone === 'America/New_York', 'complete named-zone foundation');
checkCalendarProjection(Intl.supportedValuesOf('numberingSystem').length === 78, 'complete numeric authorities');
print('intl-calendar-projection:ok');
262;
