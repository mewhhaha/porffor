function check(value, message) { if (!value) throw new Error(message); }
const epoch = 1735213600321000000n;
const zdt = new Temporal.ZonedDateTime(epoch, 'UTC');
const defaults = zdt.toLocaleString('en-US');
check(defaults.includes('2024') && defaults.includes('26') && defaults.includes('46') && defaults.includes('40'), 'default includes date and time');
check(defaults.includes('UTC') || defaults.includes('Coordinated Universal Time'), 'default includes short zone name');
check(!defaults.includes('321'), 'default excludes fractional seconds');
for (const options of [{ year: 'numeric' }, { fractionalSecondDigits: 3 }, { dateStyle: 'long' }, { timeStyle: 'short' }, { dateStyle: 'short', timeStyle: 'short' }]) {
  const expected = new Intl.DateTimeFormat('en-US', { ...options, timeZone: 'UTC' }).format(new Temporal.Instant(epoch));
  check(zdt.toLocaleString('en-US', options) === expected, 'explicit components/styles do not receive new zoned defaults');
}
const utc = new Temporal.ZonedDateTime(epoch, '+00:00').toLocaleString('en-US', { timeZoneName: 'longOffset' });
const plus = new Temporal.ZonedDateTime(epoch, '+01:00').toLocaleString('en-US', { timeZoneName: 'longOffset' });
const minus = new Temporal.ZonedDateTime(epoch, '-01:00').toLocaleString('en-US', { timeZoneName: 'longOffset' });
const expectedZero = new Intl.DateTimeFormat('en-US', { timeZone: '+00:00', timeZoneName: 'longOffset' }).format(new Temporal.Instant(epoch));
check(utc === expectedZero, 'zero numeric offset follows the ordinary Intl data service');
check(plus.includes('GMT+01:00') && minus.includes('GMT-01:00'), 'numeric identifier remains numeric Intl zone');
const alias = new Temporal.ZonedDateTime(epoch, 'Asia/Calcutta');
const primary = new Temporal.ZonedDateTime(epoch, 'Asia/Kolkata');
check(alias.toLocaleString('en-US') === primary.toLocaleString('en-US'), 'named aliases resolve to the same formatting zone');
const vienna = new Temporal.ZonedDateTime(epoch, 'Europe/Vienna');
check(vienna.toLocaleString('en-US', { timeZoneName: 'long' }).includes('Central European Standard Time'), 'long zone name is from receiver zone');
let conflict;
try { zdt.toLocaleString('en-US', { dateStyle: 'long', year: 'numeric' }); } catch (error) { conflict = error; }
check(conflict && conflict.constructor === TypeError, 'style/component conflict remains checked before injected defaults');
print('ok');
262;
