function check(value, label) { if (!value) throw new Error(label); }
const ids = Intl.supportedValuesOf('numberingSystem');
check(ids.includes('tols') && ids.length === 78, 'checked shared inventory');
check(ids.includes('hanidec') && ids.includes('mathbold'), 'old alphabets retained');
const foreign = __lilaCreateRealm().global;
const nf = new Intl.NumberFormat('en-US-u-nu-tols', { useGrouping: false });
const dtf = new Intl.DateTimeFormat('en-US-u-nu-tols', { timeZone: 'UTC', year: 'numeric' });
for (const parts of [foreign.Intl.NumberFormat.prototype.formatToParts.call(nf, 123), foreign.Intl.DateTimeFormat.prototype.formatToParts.call(dtf, 0)]) {
  check(Object.getPrototypeOf(parts) === foreign.Array.prototype, 'called function array Realm');
  for (const part of parts) check(Object.getPrototypeOf(part) === foreign.Object.prototype, 'called function part Realm');
}
const foreignIds = foreign.Intl.supportedValuesOf('numberingSystem');
check(Object.getPrototypeOf(foreignIds) === foreign.Array.prototype && foreignIds.includes('tols'), 'called enumeration Realm');
print('ok');
262;
