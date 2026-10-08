function check(value, label) { if (!value) throw new Error(label); }
for (const C of [Intl.NumberFormat, Intl.DateTimeFormat]) {
  const same = new C('en-US-u-nu-tols', { numberingSystem: 'tols' }).resolvedOptions();
  check(same.locale === 'en-US-u-nu-tols' && same.numberingSystem === 'tols', 'extension retained');
  const override = new C('en-US-u-nu-tols', { numberingSystem: 'arab' }).resolvedOptions();
  check(override.locale === 'en-US' && override.numberingSystem === 'arab', 'explicit override');
  const unsupported = new C('en-US-u-nu-tols', { numberingSystem: 'unknown' }).resolvedOptions();
  check(unsupported.numberingSystem === 'tols', 'unsupported option retains valid extension');
}
const money = new Intl.NumberFormat('en-US-u-nu-tols', { style: 'currency', currency: 'USD', currencyDisplay: 'code' });
check(money.format(123.45) === 'USD\u00a0𑷡𑷢𑷣.𑷤𑷥', 'real currency pattern and spacing');
check(money.formatToParts(123.45).some(p => p.type === 'currency' && p.value === 'USD'), 'currency text preserved');
check(new Intl.NumberFormat('en-US-u-nu-hanidec', { useGrouping: false }).format(123) === '一二三', 'old nondecimal ideographs retained');
check(new Intl.NumberFormat('en-US-u-nu-mathbold', { useGrouping: false }).format(12) === '𝟏𝟐', 'old supplementary digits retained');
print('ok');
262;
