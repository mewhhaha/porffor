function check(value, message) { if (!value) throw new Error(message); }
const options = {
  year: 'numeric', month: '2-digit', day: '2-digit',
  hour: '2-digit', minute: '2-digit', second: '2-digit',
  fractionalSecondDigits: 3, hourCycle: 'h23', timeZoneName: 'short'
};
function legacy(epochMs, zone) {
  return new Intl.DateTimeFormat('en-US', { ...options, timeZone: zone }).format(epochMs);
}
const negative = new Temporal.ZonedDateTime(-1n, 'Europe/Paris');
for (const property of ['epochNanoseconds', 'timeZoneId', 'calendarId', 'toPlainDateTime', 'valueOf']) {
  Object.defineProperty(negative, property, { get() { throw new Error('public receiver read ' + property); } });
}
const negativeOutput = negative.toLocaleString('en-US', options);
check(negativeOutput === legacy(-1, 'Europe/Paris'), 'negative epoch uses floor milliseconds and stored Paris zone');
check(negativeOutput.includes('999'), 'negative sub-millisecond instant must retain prior millisecond');
const before = new Temporal.ZonedDateTime(1710053999999999999n, 'America/New_York');
const after = new Temporal.ZonedDateTime(1710054000000000000n, 'America/New_York');
const beforeOutput = before.toLocaleString('en-US', options);
const afterOutput = after.toLocaleString('en-US', options);
check(beforeOutput === legacy(1710053999999, 'America/New_York'), 'offset belongs to pre-transition epoch');
check(afterOutput === legacy(1710054000000, 'America/New_York'), 'offset belongs to post-transition epoch');
check(beforeOutput !== afterOutput, 'transition must change actual wall time and zone output');
let ordinaryRejected = false;
try { new Intl.DateTimeFormat('en-US').format(after); } catch (error) { ordinaryRejected = error.constructor === TypeError; }
check(ordinaryRejected, 'ordinary Intl format retains ZonedDateTime rejection');
print('ok');
262;
