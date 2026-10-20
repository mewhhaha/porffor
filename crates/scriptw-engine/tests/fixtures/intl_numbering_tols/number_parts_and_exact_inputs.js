function check(value, label) { if (!value) throw new Error(label); }
const f = new Intl.NumberFormat('en-US-u-nu-tols', { useGrouping: false, maximumFractionDigits: 3 });
check(f.resolvedOptions().numberingSystem === 'tols', 'genuine selected alphabet');
check(f.format(1234567890n) === '𑷡𑷢𑷣𑷤𑷥𑷦𑷧𑷨𑷩𑷠', 'all ten supplementary digits');
check(f.format(12345678901234567890n) === '𑷡𑷢𑷣𑷤𑷥𑷦𑷧𑷨𑷩𑷠𑷡𑷢𑷣𑷤𑷥𑷦𑷧𑷨𑷩𑷠', 'exact BigInt retained');
check(f.format('-0.125') === '-𑷠.𑷡𑷢𑷥', 'exact signed decimal');
const parts = f.formatToParts('-0.125');
check(parts.map(p => p.value).join('') === '-𑷠.𑷡𑷢𑷥', 'part join');
check(parts.some(p => p.type === 'integer' && p.value === '𑷠'), 'integer role');
check(parts.some(p => p.type === 'fraction' && p.value === '𑷡𑷢𑷥'), 'fraction role');
check(parts.some(p => p.type === 'minusSign' && p.value === '-'), 'unaltered symbol');
print('ok');
262;
