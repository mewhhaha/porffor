function check(value, message) { if (!value) throw new Error(message); }
const f = new Intl.DateTimeFormat('en-US', { calendar: 'buddhist', timeZone: 'UTC', year: 'numeric', month: 'numeric', day: 'numeric' });
const foreign = __lilaCreateRealm().global;
const parts = foreign.Intl.DateTimeFormat.prototype.formatRangeToParts.call(f, 86400000, 31622400000);
check(Object.getPrototypeOf(parts) === foreign.Array.prototype, 'borrowed array Realm');
for (const p of parts) {
  check(Object.getPrototypeOf(p) === foreign.Object.prototype, 'borrowed part Realm');
  for (const key of Object.keys(p)) {
    const d = Object.getOwnPropertyDescriptor(p, key);
    check(d.writable && d.enumerable && d.configurable, 'ordinary part descriptors');
  }
}
check(parts.some(p => p.type === 'year' && p.value === '2513' && p.source === 'startRange'), 'start year');
check(parts.some(p => p.type === 'year' && p.value === '2514' && p.source === 'endRange'), 'end year');
check(parts.map(p => p.value).join('') === f.formatRange(86400000, 31622400000), 'range join');
const marker = {}, trace = [];
try {
  f.formatRange({ valueOf() { trace.push('left'); return NaN; } }, { valueOf() { trace.push('right'); throw marker; } });
  throw new Error('missing abrupt');
} catch (error) { check(error === marker, 'right completion precedes clipping error'); }
check(trace.join(',') === 'left,right', 'both conversions observed');
print('ok');
262;
