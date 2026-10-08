function check(value, message) { if (!value) throw new Error(message); }
const f = new Intl.DateTimeFormat('en-US', { calendar: 'buddhist', timeZone: 'UTC', year: 'numeric', month: 'numeric', day: 'numeric', hourCycle: 'h23', hour: '2-digit', minute: '2-digit', second: '2-digit', fractionalSecondDigits: 3 });
const negative = f.formatToParts(new Temporal.Instant(-1n));
for (const [type, value] of [['year', '2512'], ['day', '31'], ['hour', '23'], ['minute', '59'], ['second', '59'], ['fractionalSecond', '999']]) {
  check(negative.some(p => p.type === type && p.value === value), 'negative normalized epoch: ' + type);
}
const clipped = f.formatToParts(-0.1);
check(clipped.some(p => p.type === 'year' && p.value === '2513'), 'legacy TimeClip remains distinct');
check(clipped.some(p => p.type === 'fractionalSecond' && p.value === '000'), 'legacy truncated fraction');
for (const [ns, year, day] of [[-8640000000000000000000n, '271279', '20'], [8640000000000000000000n, '276303', '13']]) {
  const input = new Temporal.Instant(ns), parts = f.formatToParts(input);
  check(parts.some(p => p.type === 'year' && p.value === year), 'terminal year');
  check(parts.some(p => p.type === 'day' && p.value === day), 'terminal day');
  check(parts.map(p => p.value).join('') === f.format(input), 'terminal parts join');
}
print('ok');
262;
