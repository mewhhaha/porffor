function check(value, message) { if (!value) throw new Error(message); }
const selected = new Intl.DateTimeFormat('en-US-u-ca-buddhist', { timeZone: 'UTC', year: 'numeric', era: 'short' });
check(selected.resolvedOptions().calendar === 'buddhist', 'extension selection');
check(selected.resolvedOptions().locale === 'en-US-u-ca-buddhist', 'retained extension');
const overridden = new Intl.DateTimeFormat('en-US-u-ca-buddhist', { calendar: 'gregory', timeZone: 'UTC', year: 'numeric' });
check(overridden.resolvedOptions().calendar === 'gregory', 'explicit precedence');
check(overridden.resolvedOptions().locale === 'en-US', 'overridden extension removed');
for (const [year, numeric, twoDigit] of [[-543, '1', '01'], [-544, '2', '02'], [-643, '101', '01']]) {
  const date = new Temporal.PlainDate(year, 1, 2);
  for (const [width, expected] of [['numeric', numeric], ['2-digit', twoDigit]]) {
    const f = new Intl.DateTimeFormat('en-US', { calendar: 'buddhist', timeZone: 'UTC', year: width, era: 'short' });
    const parts = f.formatToParts(date);
    check(parts.some(p => p.type === 'year' && p.value === expected), 'nonpositive year before width formatting');
    check(parts.some(p => p.type === 'era' && p.value === 'BE'), 'one era at and before Buddhist year zero');
  }
}
print('ok');
262;
