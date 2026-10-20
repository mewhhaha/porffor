function check(value, label) { if (!value) throw new Error(label); }
const f = new Intl.DateTimeFormat('en-US-u-nu-tols', {
  timeZone: 'UTC', hourCycle: 'h23', hour: '2-digit', minute: '2-digit', second: '2-digit', fractionalSecondDigits: 3
});
check(f.resolvedOptions().numberingSystem === 'tols', 'DTF actual alphabet');
check(f.format(1704076506789) === '𑷠𑷢:𑷣𑷥:𑷠𑷦.𑷧𑷨𑷩', 'time, padding and fraction');
const parts = f.formatToParts(1704076506789);
check(parts.some(p => p.type === 'fractionalSecond' && p.value === '𑷧𑷨𑷩'), 'fraction role');
check(parts.map(p => p.value).join('') === f.format(1704076506789), 'scalar join');
const b = new Intl.DateTimeFormat('en-US-u-ca-buddhist-nu-tols', { timeZone: 'UTC', year: 'numeric', month: 'numeric', day: 'numeric' });
const bp = b.formatToParts(new Temporal.PlainDate(2024, 2, 29, 'buddhist'));
check(bp.some(p => p.type === 'year' && p.value === '𑷢𑷥𑷦𑷧'), 'real Buddhist solar projection');
const range = f.formatRangeToParts(123, 456);
check(range.some(p => p.type === 'fractionalSecond' && p.value === '𑷡𑷢𑷣' && p.source === 'startRange'), 'start fraction');
check(range.some(p => p.type === 'fractionalSecond' && p.value === '𑷤𑷥𑷦' && p.source === 'endRange'), 'end fraction');
check(range.map(p => p.value).join('') === f.formatRange(123, 456), 'range join');
print('ok');
262;
