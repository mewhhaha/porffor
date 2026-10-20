const calendars = Intl.supportedValuesOf('calendar');
const again = Intl.supportedValuesOf('calendar');
const descriptor = Object.getOwnPropertyDescriptor(calendars, '0');
if (!calendars.includes('gregory') || calendars === again ||
    !descriptor.writable || !descriptor.enumerable || !descriptor.configurable) {
  throw 'selected supported-values catalogue or fresh Array descriptor';
}
if (new Intl.Locale('iw-IL').toString() !== 'he-IL' ||
    new Intl.Locale('en-US').getCalendars()[0] !== 'gregory' ||
    new Intl.NumberFormat('en-US').format(1234) !== '1,234' ||
    new Intl.ListFormat('en').format(['A', 'B']) !== 'A and B') {
  throw 'selected Locale, NumberFormat or ListFormat operation';
}
print('intl-profile:ok');
262;
